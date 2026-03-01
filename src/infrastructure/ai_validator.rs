use anyhow::{Context, Result};
use async_openai::{
    config::OpenAIConfig,
    types::{
        chat::{
            ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
            ChatCompletionRequestUserMessage, CreateChatCompletionRequest,
        },
        embeddings::CreateEmbeddingRequestArgs,
    },
    Client,
};
use async_trait::async_trait;

use crate::domain::ports::{AIValidator, EmbeddingService, ValidationMethod, ValidationResult};

/// OpenAI-based AI validator with cascading validation strategy
pub struct OpenAIValidator {
    client: Client<OpenAIConfig>,
    embedding_model: String,
    chat_model: String,
    _exact_match_threshold: f32,
    embedding_threshold: f32,
}

impl OpenAIValidator {
    pub fn new(api_key: String) -> Self {
        let config = OpenAIConfig::new().with_api_key(api_key);
        let client = Client::with_config(config);

        Self {
            client,
            embedding_model: "text-embedding-3-small".to_string(),
            chat_model: "gpt-4o-mini".to_string(),
            _exact_match_threshold: 0.95,
            embedding_threshold: 0.85,
        }
    }

    /// Check for exact match (case-insensitive, trimmed)
    fn check_exact_match(&self, expected: &str, user_answer: &str) -> Option<f32> {
        let expected_normalized = expected.trim().to_lowercase();
        let user_normalized = user_answer.trim().to_lowercase();

        if expected_normalized == user_normalized {
            Some(1.0)
        } else {
            None
        }
    }

    /// Calculate similarity using OpenAI embeddings
    async fn check_embedding_similarity(&self, expected: &str, user_answer: &str) -> Result<f32> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.embedding_model)
            .input(vec![expected.to_string(), user_answer.to_string()])
            .build()?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await?;

        if response.data.len() < 2 {
            return Ok(0.0);
        }

        let embedding1 = &response.data[0].embedding;
        let embedding2 = &response.data[1].embedding;

        // Calculate cosine similarity
        let similarity = cosine_similarity(embedding1, embedding2);
        Ok(similarity)
    }

    /// Validate using LLM
    async fn check_llm_validation(
        &self,
        expected: &str,
        user_answer: &str,
        question: &str,
    ) -> Result<f32> {
        let system_prompt = r#"You are an expert language tutor evaluating student answers.
Compare the student's answer with the expected answer in the context of the question.
Rate the answer from 0.0 to 1.0 based on semantic correctness and completeness.
Consider:
- Meaning and intent (more important than exact wording)
- Grammatical correctness
- Completeness of the response

Respond with ONLY a number between 0.0 and 1.0, nothing else."#;

        let user_prompt = format!(
            "Question: {}\n\nExpected Answer: {}\n\nStudent Answer: {}\n\nScore:",
            question, expected, user_answer
        );

        let request = CreateChatCompletionRequest {
            model: self.chat_model.clone(),
            messages: vec![
                ChatCompletionRequestMessage::System(
                    ChatCompletionRequestSystemMessage {
                        content: system_prompt.into(),
                        name: None,
                    },
                ),
                ChatCompletionRequestMessage::User(
                    ChatCompletionRequestUserMessage {
                        content: user_prompt.into(),
                        name: None,
                    },
                ),
            ],
            temperature: Some(0.0),
            max_completion_tokens: Some(10),
            ..Default::default()
        };

        let response = self.client.chat().create(request).await?;

        let score_text = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .context("No response from LLM")?;

        let score: f32 = score_text.trim().parse().unwrap_or(0.0);
        Ok(score.clamp(0.0, 1.0))
    }
}

#[async_trait]
impl AIValidator for OpenAIValidator {
    async fn validate(
        &self,
        expected_answer: &str,
        user_answer: &str,
        question_context: &str,
    ) -> Result<ValidationResult> {
        // Strategy 1: Exact match
        if let Some(score) = self.check_exact_match(expected_answer, user_answer) {
            return Ok(ValidationResult {
                score,
                method: ValidationMethod::Exact,
            });
        }

        // Strategy 2: Embedding similarity
        match self
            .check_embedding_similarity(expected_answer, user_answer)
            .await
        {
            Ok(score) if score >= self.embedding_threshold => {
                return Ok(ValidationResult {
                    score,
                    method: ValidationMethod::Embedding,
                });
            }
            Ok(score) if score >= 0.6 => {
                // Borderline case - use LLM for final decision
                tracing::info!(
                    "Embedding score borderline ({}), falling back to LLM",
                    score
                );
            }
            Err(e) => {
                tracing::warn!("Embedding check failed: {}, falling back to LLM", e);
            }
            _ => {}
        }

        // Strategy 3: LLM validation (most expensive)
        let score = self
            .check_llm_validation(expected_answer, user_answer, question_context)
            .await?;

        Ok(ValidationResult {
            score,
            method: ValidationMethod::Llm,
        })
    }
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}

// ---------------------------------------------------------------------------
// Fallback validator (no OpenAI dependency ? used when key is not configured)
// ---------------------------------------------------------------------------

/// Simple heuristic validator that works without an OpenAI API key.
/// Uses exact match and word-overlap (Jaccard) similarity.
/// Suitable for development / when OPENAI_API_KEY is not set.
pub struct FallbackValidator;

#[async_trait]
impl AIValidator for FallbackValidator {
    async fn validate(
        &self,
        expected_answer: &str,
        user_answer: &str,
        _question_context: &str,
    ) -> Result<ValidationResult> {
        let expected = expected_answer.trim().to_lowercase();
        let actual = user_answer.trim().to_lowercase();

        // Exact match
        if expected == actual {
            return Ok(ValidationResult {
                score: 1.0,
                method: ValidationMethod::Exact,
            });
        }

        // Word-overlap (Jaccard)
        let expected_words: std::collections::HashSet<&str> = expected.split_whitespace().collect();
        let actual_words: std::collections::HashSet<&str> = actual.split_whitespace().collect();

        let intersection = expected_words.intersection(&actual_words).count() as f32;
        let union = expected_words.union(&actual_words).count() as f32;

        let jaccard = if union > 0.0 {
            intersection / union
        } else {
            0.0
        };

        // Scale: 0.0?0.49 ? Again/Hard, 0.5?0.89 ? Good, 0.9?1.0 ? Easy
        Ok(ValidationResult {
            score: jaccard,
            method: ValidationMethod::Exact, // closest approximation
        })
    }
}

#[async_trait]
impl EmbeddingService for OpenAIValidator {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.embedding_model)
            .input(text)
            .build()?;

        let response = self
            .client
            .embeddings()
            .create(request)
            .await
            .context("Failed to generate embedding")?;

        if response.data.is_empty() {
            anyhow::bail!("No embedding returned from API");
        }

        Ok(response.data[0].embedding.clone())
    }
}

#[async_trait]
impl EmbeddingService for FallbackValidator {
    async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>> {
        anyhow::bail!("Embedding generation not available without OPENAI_API_KEY")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![1.0, 0.0];
        let d = vec![0.0, 1.0];
        assert!((cosine_similarity(&c, &d) - 0.0).abs() < 0.001);
    }
}

use anyhow::Result;
use async_trait::async_trait;

/// AI Validator trait - defines the interface for AI-based answer validation
#[async_trait]
pub trait AIValidator: Send + Sync {
    /// Validates a user's answer against the expected answer
    /// Returns a score between 0.0 and 1.0, and the validation method used
    async fn validate(
        &self,
        expected_answer: &str,
        user_answer: &str,
        question_context: &str,
    ) -> Result<ValidationResult>;
}

/// Embedding Service trait - generates embeddings for text
#[async_trait]
pub trait EmbeddingService: Send + Sync {
    /// Generates an embedding vector for the given text
    /// Returns a vector of floats representing the text in semantic space
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;
}

/// Result of AI validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub score: f32,
    pub method: ValidationMethod,
}

/// Method used for validation
#[derive(Debug, Clone)]
pub enum ValidationMethod {
    Exact,
    Embedding,
    Llm,
}

impl ValidationMethod {
    pub fn as_str(&self) -> &str {
        match self {
            ValidationMethod::Exact => "exact",
            ValidationMethod::Embedding => "embedding",
            ValidationMethod::Llm => "llm",
        }
    }
}

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::application::dtos::*;
use crate::presentation::router::AppServices;

/// Health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}

/// Create user handler
pub async fn create_user(
    State(services): State<AppServices>,
    Json(req): Json<CreateUserRequest>,
) -> Response {
    match services.user_service.create_user(req).await {
        Ok(user) => (StatusCode::CREATED, Json(user)).into_response(),
        Err(err) => err.into_response(),
    }
}

/// Get user handler
pub async fn get_user(Path(user_id): Path<Uuid>, State(services): State<AppServices>) -> Response {
    match services.user_service.get_user(user_id).await {
        Ok(user) => Json(user).into_response(),
        Err(err) => err.into_response(),
    }
}

/// Create card handler
pub async fn create_card(
    Path(user_id): Path<Uuid>,
    State(services): State<AppServices>,
    Json(req): Json<CreateCardRequest>,
) -> Response {
    match services.card_service.create_card(user_id, req).await {
        Ok(card) => (StatusCode::CREATED, Json(card)).into_response(),
        Err(err) => err.into_response(),
    }
}

/// Get user cards handler
pub async fn get_user_cards(
    Path(user_id): Path<Uuid>,
    State(services): State<AppServices>,
) -> Response {
    match services.card_service.get_user_cards(user_id).await {
        Ok(cards) => Json(cards).into_response(),
        Err(err) => err.into_response(),
    }
}

/// Submit review handler
pub async fn submit_review(
    Path((user_id, card_id)): Path<(Uuid, Uuid)>,
    State(services): State<AppServices>,
    Json(req): Json<LegacyReviewCardRequest>,
) -> Response {
    match services
        .review_service
        .submit_review(card_id, user_id, req)
        .await
    {
        Ok(review) => (StatusCode::CREATED, Json(review)).into_response(),
        Err(err) => err.into_response(),
    }
}

/// Submit intelligent review with AI validation (API v1)
/// POST /api/v1/reviews
/// Body: { "card_id": "uuid", "user_id": "uuid", "user_answer": "string" }
pub async fn submit_intelligent_review(
    State(services): State<AppServices>,
    Json(req): Json<SubmitReviewRequest>,
) -> Response {
    match services
        .review_card_use_case
        .execute(req.card_id, req.user_id, req.user_answer)
        .await
    {
        Ok(result) => {
            let response = ReviewResponseDto {
                card_id: result.card_id,
                ai_score: result.ai_score,
                fsrs_rating: result.fsrs_rating,
                validation_method: result.validation_method.as_str().to_string(),
                next_review_in_days: result.next_review_in_days,
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(err) => {
            tracing::error!("Review failed: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": format!("Review failed: {}", err)
                })),
            )
                .into_response()
        }
    }
}

/// Submit review request for API v1
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubmitReviewRequest {
    pub card_id: Uuid,
    pub user_id: Uuid,
    pub user_answer: String,
}

/// Create deck handler
pub async fn create_deck(
    Path(user_id): Path<Uuid>,
    State(services): State<AppServices>,
    Json(req): Json<CreateDeckRequest>,
) -> Response {
    match services.deck_service.create_deck(user_id, req).await {
        Ok(deck) => (StatusCode::CREATED, Json(deck)).into_response(),
        Err(err) => err.into_response(),
    }
}

/// Get user decks handler
pub async fn get_user_decks(
    Path(user_id): Path<Uuid>,
    State(services): State<AppServices>,
) -> Response {
    match services.deck_service.get_user_decks(user_id).await {
        Ok(decks) => Json(decks).into_response(),
        Err(err) => err.into_response(),
    }
}

/// Get cards by deck handler
pub async fn get_deck_cards(
    Path(deck_id): Path<Uuid>,
    State(services): State<AppServices>,
) -> Response {
    match services.card_service.get_deck_cards(deck_id).await {
        Ok(cards) => Json(cards).into_response(),
        Err(err) => err.into_response(),
    }
}

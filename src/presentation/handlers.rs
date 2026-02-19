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
pub async fn get_user(
    Path(user_id): Path<Uuid>,
    State(services): State<AppServices>,
) -> Response {
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
    Json(req): Json<ReviewCardRequest>,
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

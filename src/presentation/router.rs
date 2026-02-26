use axum::{
    routing::{delete, get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use super::handlers::*;
use crate::application::{
    services::{CardService, DeckService, ReviewService, UserService},
    use_cases::{GetDeckStatsUseCase, GetUserStatsUseCase, ReviewCardUseCase},
};
use crate::domain::ports::AIValidator;
use crate::domain::repositories::{CardRepository, ReviewLogRepository};

/// Container for application services
#[derive(Clone)]
pub struct AppServices {
    pub user_service: Arc<UserService>,
    pub card_service: Arc<CardService>,
    pub deck_service: Arc<DeckService>,
    pub review_service: Arc<ReviewService>,
    pub review_card_use_case: Arc<dyn ReviewCardUseCaseTrait>,
    pub get_user_stats_use_case: Arc<GetUserStatsUseCase>,
    pub get_deck_stats_use_case: Arc<GetDeckStatsUseCase>,
}

/// Trait to allow dynamic dispatch for ReviewCardUseCase
#[async_trait::async_trait]
pub trait ReviewCardUseCaseTrait: Send + Sync {
    async fn execute(
        &self,
        card_id: uuid::Uuid,
        user_id: uuid::Uuid,
        user_answer: String,
    ) -> anyhow::Result<crate::application::use_cases::ReviewResult>;
}

/// Blanket implementation for any ReviewCardUseCase
#[async_trait::async_trait]
impl<R, L, V> ReviewCardUseCaseTrait for ReviewCardUseCase<R, L, V>
where
    R: CardRepository + 'static,
    L: ReviewLogRepository + 'static,
    V: AIValidator + 'static,
{
    async fn execute(
        &self,
        card_id: uuid::Uuid,
        user_id: uuid::Uuid,
        user_answer: String,
    ) -> anyhow::Result<crate::application::use_cases::ReviewResult> {
        self.execute(card_id, user_id, user_answer).await
    }
}

/// Create the main router with all endpoints
pub fn create_router(app_services: AppServices) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // User routes
        .route("/users", post(create_user))
        .route("/users/{user_id}", get(get_user))
        // Deck routes
        .route(
            "/users/{user_id}/decks",
            post(create_deck).get(get_user_decks),
        )
        .route(
            "/users/{user_id}/decks/{deck_id}",
            delete(delete_deck),
        )
        .route("/decks/{deck_id}/cards", get(get_deck_cards))
        // Card routes
        .route(
            "/users/{user_id}/cards",
            post(create_card).get(get_user_cards),
        )
        .route(
            "/users/{user_id}/cards/{card_id}",
            delete(delete_card),
        )
        // Review routes (legacy)
        .route(
            "/users/{user_id}/cards/{card_id}/reviews",
            post(submit_review),
        )
        // API v1 routes (new intelligent review)
        .route("/api/v1/reviews", post(submit_intelligent_review))
        // Statistics routes
        .route("/api/v1/users/{user_id}/stats", get(get_user_stats))
        .route("/api/v1/decks/{deck_id}/stats", get(get_deck_stats))
        // Middleware
        .with_state(app_services)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

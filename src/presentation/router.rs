use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use super::handlers::*;
use crate::application::{
    services::{CardService, ReviewService, UserService},
};

/// Container for application services
#[derive(Clone)]
pub struct AppServices {
    pub user_service: Arc<UserService>,
    pub card_service: Arc<CardService>,
    pub review_service: Arc<ReviewService>,
}

/// Create the main router with all endpoints
pub fn create_router(app_services: AppServices) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // User routes
        .route("/users", post(create_user))
        .route("/users/{user_id}", get(get_user))
        
        // Card routes
        .route("/users/{user_id}/cards", post(create_card).get(get_user_cards))
        
        // Review routes
        .route("/users/{user_id}/cards/{card_id}/reviews", post(submit_review))
        
        // Middleware
        .with_state(app_services)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}

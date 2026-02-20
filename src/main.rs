use re_mem::{
    application::services::{CardService, ReviewService, UserService},
    application::use_cases::ReviewCardUseCase,
    infrastructure::{
        database::{init_db_pool, DbConfig},
        repositories::{PgCardRepository, PgReviewRepository, PgReviewLogRepository, PgUserRepository},
        ai_validator::OpenAIValidator,
    },
    presentation::router::{create_router, AppServices},
    shared::event_bus::EventBus,
};
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Load configuration
    dotenv::dotenv().ok();
    let db_config = DbConfig::from_env();

    // Initialize database
    let db_pool = match init_db_pool(&db_config).await {
        Ok(pool) => {
            tracing::info!("Database connected successfully");
            pool
        }
        Err(e) => {
            tracing::error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize repositories
    let user_repo = Arc::new(PgUserRepository::new(db_pool.clone()));
    let card_repo = Arc::new(PgCardRepository::new(db_pool.clone()));
    let review_repo = Arc::new(PgReviewRepository::new(db_pool.clone()));
    let review_log_repo = Arc::new(PgReviewLogRepository::new(db_pool.clone()));

    // Initialize application services (legacy)
    let user_service = Arc::new(UserService::new(user_repo));
    let card_service = Arc::new(CardService::new(card_repo.clone()));
    let review_service = Arc::new(ReviewService::new(review_repo));

    // Initialize AI Validator
    let openai_api_key = std::env::var("OPENAI_API_KEY")
        .unwrap_or_else(|_| {
            tracing::warn!("OPENAI_API_KEY not set, using placeholder");
            "sk-placeholder".to_string()
        });
    let ai_validator = Arc::new(OpenAIValidator::new(openai_api_key));

    // Initialize Event Bus
    let event_bus = Arc::new(EventBus::new());

    // Initialize Review Card Use Case (new intelligent review)
    let review_card_use_case = Arc::new(ReviewCardUseCase::new(
        card_repo,
        review_log_repo,
        ai_validator,
        event_bus,
    ));

    let app_services = AppServices {
        user_service,
        card_service,
        review_service,
        review_card_use_case,
    };

    // Create router
    let app = create_router(app_services);

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Server starting on 0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}

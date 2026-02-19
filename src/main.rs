use re_mem::{
    application::services::{CardService, ReviewService, UserService},
    infrastructure::{
        database::{init_db_pool, DbConfig},
        repositories::{PgCardRepository, PgReviewRepository, PgUserRepository},
    },
    presentation::router::{create_router, AppServices},
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
    let review_repo = Arc::new(PgReviewRepository::new(db_pool));

    // Initialize application services
    let user_service = Arc::new(UserService::new(user_repo));
    let card_service = Arc::new(CardService::new(card_repo));
    let review_service = Arc::new(ReviewService::new(review_repo));

    let app_services = AppServices {
        user_service,
        card_service,
        review_service,
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

use re_mem::{
    application::services::{AuthService, CardService, DeckService, ReviewService, UserService},
    application::use_cases::{GetDeckStatsUseCase, GetUserStatsUseCase, ImportAnkiUseCase, ImportTsvUseCase, ReviewCardUseCase},
    infrastructure::{
        ai_validator::{FallbackValidator, OpenAIValidator},
        database::{init_db_pool, DbConfig},
        repositories::{
            PgCardRepository, PgDeckRepository, PgDeckStatsRepository, PgReviewLogRepository,
            PgReviewRepository, PgUserRepository, PgUserStatsRepository,
        },
        StatisticsEventHandler,
    },
    domain::{
        ports::EmbeddingService,
        repositories::{CardRepository, DeckRepository, DeckStatsRepository},
    },
    presentation::router::{create_router, AppServices, ReviewCardUseCaseTrait},
    shared::event_bus::EventBus,
};
use std::sync::Arc;

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
    let deck_repo = Arc::new(PgDeckRepository::new(db_pool.clone()));
    let review_repo = Arc::new(PgReviewRepository::new(db_pool.clone()));
    let review_log_repo = Arc::new(PgReviewLogRepository::new(db_pool.clone()));
    let user_stats_repo = Arc::new(PgUserStatsRepository::new(db_pool.clone()));
    let deck_stats_repo = Arc::new(PgDeckStatsRepository::new(db_pool.clone()));

    // Initialize Event Bus and register handlers
    let mut event_bus = EventBus::new();
    
    // Initialize Statistics Event Handler
    let stats_handler = Arc::new(StatisticsEventHandler::new(
        user_stats_repo.clone(),
        deck_stats_repo.clone(),
        card_repo.clone(),
    ));
    
    // Register the statistics handler
    event_bus.register_handler(stats_handler);
    
    let event_bus = Arc::new(event_bus);

    // Initialize application services (legacy)
    let user_service = Arc::new(UserService::new(user_repo));
    let card_service = Arc::new(CardService::new(card_repo.clone(), event_bus.clone()));
    let deck_service = Arc::new(DeckService::new(deck_repo.clone()));
    let review_service = Arc::new(ReviewService::new(review_repo));

    // Initialize statistics use cases
    let get_user_stats_use_case = Arc::new(GetUserStatsUseCase::new(user_stats_repo.clone()));
    let get_deck_stats_use_case =
        Arc::new(GetDeckStatsUseCase::new(deck_stats_repo.clone(), deck_repo));

    // Initialize AI Validator and Review Card Use Case
    let (review_card_use_case, embedding_service): (
        Arc<dyn ReviewCardUseCaseTrait>,
        Arc<dyn EmbeddingService>,
    ) = match std::env::var("OPENAI_API_KEY") {
        Ok(api_key) => {
            tracing::info!("Using OpenAI validator");
            let validator = Arc::new(OpenAIValidator::new(api_key));
            let embedding: Arc<dyn EmbeddingService> = validator.clone();
            let uc = Arc::new(ReviewCardUseCase::new(
                card_repo.clone(),
                review_log_repo,
                validator,
                event_bus,
            )) as Arc<dyn ReviewCardUseCaseTrait>;
            (uc, embedding)
        }
        Err(_) => {
            tracing::warn!(
                "OPENAI_API_KEY not set — using FallbackValidator (word-overlap scoring)"
            );
            let validator = Arc::new(FallbackValidator);
            let embedding: Arc<dyn EmbeddingService> = Arc::new(FallbackValidator);
            let uc = Arc::new(ReviewCardUseCase::new(
                card_repo.clone(),
                review_log_repo,
                validator,
                event_bus,
            )) as Arc<dyn ReviewCardUseCaseTrait>;
            (uc, embedding)
        }
    };

    // Import use cases (cast concrete repos to trait objects)
    let card_repo_dyn: Arc<dyn CardRepository> = card_repo.clone();
    let deck_repo_dyn: Arc<dyn DeckRepository> = Arc::new(PgDeckRepository::new(db_pool.clone()));
    let deck_stats_repo_dyn: Arc<dyn DeckStatsRepository> = deck_stats_repo.clone();

    let import_tsv_use_case = Arc::new(ImportTsvUseCase::new(
        card_repo_dyn.clone(),
        deck_stats_repo_dyn.clone(),
        embedding_service.clone(),
    ));
    let import_anki_use_case = Arc::new(ImportAnkiUseCase::new(
        card_repo_dyn,
        deck_repo_dyn,
        deck_stats_repo_dyn,
        embedding_service,
    ));

    // Initialize auth service
    let auth_service = Arc::new(AuthService::new(
        Arc::new(PgUserRepository::new(db_pool.clone())),
    ));

    let app_services = AppServices {
        user_service,
        card_service,
        deck_service,
        review_service,
        review_card_use_case,
        get_user_stats_use_case,
        get_deck_stats_use_case,
        auth_service,
        import_tsv_use_case,
        import_anki_use_case,
    };

    // Create router
    let app = create_router(app_services);

    // Run server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Server starting on 0.0.0.0:3000");

    axum::serve(listener, app).await.expect("Server failed");
}

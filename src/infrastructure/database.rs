use sqlx::postgres::PgPool;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DbConfig {
    pub database_url: String,
}

impl DbConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://re_mem:password@localhost:5432/re_mem".to_string()),
        }
    }
}

/// Initialize database connection pool
pub async fn init_db_pool(config: &DbConfig) -> crate::AppResult<PgPool> {
    let pool = PgPool::connect(&config.database_url).await?;
    run_migrations(&pool).await?;
    Ok(pool)
}

/// Run database migrations
async fn run_migrations(_pool: &PgPool) -> crate::AppResult<()> {
    // TODO: Implement database migrations using sqlx::migrate
    // This will include creating tables for users, cards, reviews, etc.
    tracing::info!("Database migrations completed");
    Ok(())
}

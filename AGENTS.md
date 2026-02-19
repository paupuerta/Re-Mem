# ReMem Backend - AGENTS.md

This document provides comprehensive guidance for AI agents working on the ReMem backend project.

## Project Overview

ReMem is a language-learning MVP backend built with Rust using:
- **Hexagonal Architecture** (Ports & Adapters pattern)
- **FSRS** (Free Spaced Repetition Scheduler) for optimal review scheduling
- **AI-based answer checking** for intelligent assessment
- **Memory Bus Events** as foundation for future DDD migration
- **REST API** via OpenAPI/Swagger
- **PostgreSQL** for persistence
- **Docker/Kubernetes** for deployment

## Code Architecture

The project is organized into distinct layers following Hexagonal Architecture:

```
src/
├── domain/              # Core business logic (entities, value objects, repositories)
├── application/         # Use cases and application services
├── infrastructure/      # Database and external service implementations
├── presentation/        # HTTP API handlers and routing
└── shared/              # Event bus, error handling, cross-cutting concerns
```

### Layer Responsibilities

#### 1. **Domain Layer** (`src/domain/`)
- **Contains**: Entities, Value Objects, Repository interfaces, Domain Events
- **Purpose**: Encapsulates pure business logic
- **Examples**: `User`, `Card`, `Review` entities
- **Rules**:
  - No external dependencies
  - Pure Rust data structures
  - Business rules enforced here
  - Repository traits define contracts only

#### 2. **Application Layer** (`src/application/`)
- **Contains**: Use cases, Application Services, DTOs
- **Purpose**: Orchestrates domain logic for specific use cases
- **Examples**: `UserService`, `CardService`, `ReviewService`
- **Rules**:
  - Depends on domain layer only
  - Handles cross-domain operations
  - Converts between domain and presentation layers

#### 3. **Infrastructure Layer** (`src/infrastructure/`)
- **Contains**: Repository implementations, Database access, External APIs
- **Purpose**: Technical implementations of domain contracts
- **Examples**: `PgUserRepository`, `PgCardRepository`, `DbConfig`
- **Rules**:
  - Implements domain repository traits
  - Database-specific logic
  - External service integrations

#### 4. **Presentation Layer** (`src/presentation/`)
- **Contains**: HTTP handlers, routing, request/response mapping
- **Purpose**: Exposes API endpoints
- **Examples**: REST endpoints, request validation
- **Rules**:
  - Thin layer - minimal logic
  - Uses application services
  - Converts DTOs to domain objects and vice versa

#### 5. **Shared Layer** (`src/shared/`)
- **Contains**: Event bus, error handling, logging configuration
- **Purpose**: Cross-cutting concerns
- **Examples**: `AppError`, `EventBus`, `DomainEvent`
- **Rules**:
  - Reusable by all layers
  - No business logic
  - Infrastructure for other modules

## Key Principles

### SOLID Principles
- **S**ingle Responsibility: Each module has one reason to change
- **O**pen/Closed: Open for extension, closed for modification
- **L**iskov Substitution: Trait implementations are substitutable
- **I**nterface Segregation: Focused, narrow trait definitions
- **D**ependency Inversion: Depend on abstractions (traits), not concrete types

### KISS (Keep It Simple, Stupid)
- Prefer simple solutions over complex ones
- Each function should do one thing well
- Avoid over-abstraction until needed

### YAGNI (You Aren't Gonna Need It)
- Don't add features until they're actually needed
- No speculative code
- Minimal but complete implementation

## Common Tasks

### Adding a New Entity

1. **Define in Domain** (`src/domain/entities.rs`):
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
   pub struct YourEntity {
       pub id: Uuid,
       pub field: String,
       pub created_at: DateTime<Utc>,
   }
   ```

2. **Create Repository Interface** (`src/domain/repositories.rs`):
   ```rust
   #[async_trait]
   pub trait YourEntityRepository: Send + Sync {
       async fn create(&self, entity: &YourEntity) -> AppResult<Uuid>;
       async fn find_by_id(&self, id: Uuid) -> AppResult<Option<YourEntity>>;
   }
   ```

3. **Implement in Infrastructure** (`src/infrastructure/repositories.rs`):
   ```rust
   #[async_trait]
   impl YourEntityRepository for PgYourEntityRepository {
       async fn create(&self, entity: &YourEntity) -> AppResult<Uuid> {
           sqlx::query!(...).fetch_one(&self.pool).await
       }
   }
   ```

4. **Create Application Service** (`src/application/services.rs`):
   ```rust
   pub struct YourEntityService {
       repo: Arc<dyn YourEntityRepository>,
   }
   ```

5. **Add API Endpoint** (`src/presentation/handlers.rs`):
   ```rust
   pub async fn create_your_entity(
       Json(req): Json<CreateYourEntityRequest>,
       services: Extension<Arc<AppServices>>,
   ) -> AppResult<(StatusCode, Json<YourEntityDto>)> { ... }
   ```

### Adding a Database Migration

1. Create migration file: `sqlx migrate add -r <name>`
2. Edit `.sql` files in `migrations/` directory
3. Implement in `src/infrastructure/database.rs` run_migrations function

### Adding an Event

1. **Create Domain Event** in `src/shared/event_bus.rs`:
   ```rust
   pub struct YourEntityCreatedEvent {
       pub entity_id: Uuid,
       pub timestamp: DateTime<Utc>,
   }
   impl DomainEvent for YourEntityCreatedEvent { ... }
   ```

2. **Publish in Service**:
   ```rust
   event_bus.publish(YourEntityCreatedEvent { ... }).await?;
   ```

3. **Handle in Subscriber** (future DDD implementation)

## Testing Strategy

### Unit Tests
- Test value objects (`src/domain/value_objects.rs`)
- Test business logic in entities
- Located near implementation with `#[cfg(test)]` sections

### Integration Tests
- Test repository implementations
- Located in `tests/` directory
- Use test database

### Example:
```rust
#[tokio::test]
async fn test_create_user() {
    let repo = create_test_repo().await;
    let user = User::new("test@example.com".into(), "Test".into());
    let id = repo.create(&user).await.unwrap();
    assert!(repo.find_by_id(id).await.unwrap().is_some());
}
```

## Error Handling

All operations return `AppResult<T>` which is `Result<T, AppError>`:

```rust
pub enum AppError {
    ValidationError(String),
    NotFound(String),
    DatabaseError(sqlx::Error),
    // ... more variants
}
```

- Use `?` operator for error propagation
- Convert external errors via `into()`
- Handlers automatically convert `AppError` to HTTP responses

## Dependency Injection

Services are injected via `Arc<AppServices>` in handlers:

```rust
pub struct AppServices {
    pub user_service: Arc<UserService>,
    pub card_service: Arc<CardService>,
    pub review_service: Arc<ReviewService>,
}
```

Initialized in `src/main.rs` before creating router.

## Event Bus (Future DDD)

The `EventBus` in `src/shared/event_bus.rs` is designed to evolve into a full event sourcing system:

- Currently in-memory (TODO: persistence)
- Supports domain event publishing
- Ready for event handlers and subscribers
- Will support event replay for DDD

## Database Schema

Expected tables (to be created via migrations):

- `users` - User accounts
- `cards` - Flashcards with questions/answers
- `reviews` - Study session records (used by FSRS)
- Additional tables for FSRS state tracking

## Running the Project

```bash
# Development
cargo run

# Tests
cargo test

# Build
cargo build --release

# Docker
docker-compose up
```

## Linting and Formatting

```bash
# Format code
cargo fmt

# Check with clippy
cargo clippy -- -D warnings

# Full test suite
cargo test --all
```

## File Structure Reference

```
re-mem/
├── src/
│   ├── main.rs                 # Entry point
│   ├── lib.rs                  # Library root
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── entities.rs        # Domain models
│   │   ├── repositories.rs    # Repository traits
│   │   └── value_objects.rs   # Value objects
│   ├── application/
│   │   ├── mod.rs
│   │   ├── dtos.rs            # Request/Response objects
│   │   ├── services.rs        # Application services
│   │   └── use_cases.rs       # Use case definitions
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── database.rs        # DB configuration
│   │   └── repositories.rs    # Repository implementations
│   ├── presentation/
│   │   ├── mod.rs
│   │   ├── handlers.rs        # HTTP handlers
│   │   └── router.rs          # Route configuration
│   └── shared/
│       ├── mod.rs
│       ├── error.rs           # Error types
│       └── event_bus.rs       # Event bus
├── tests/
│   └── integration_tests.rs   # Integration tests
├── .docker/
│   └── Dockerfile
├── k8s/
│   └── deployment.yml         # Kubernetes manifests
├── docs/
│   ├── ARCHITECTURE.md
│   ├── API.md
│   └── DATABASE.md
├── Cargo.toml                 # Dependencies
└── docker-compose.yml         # Local development
```

## Common Patterns

### Error Handling Pattern
```rust
pub async fn operation(&self) -> AppResult<Result> {
    do_something().await?
        .ok_or_else(|| AppError::NotFound("Item".into()))?;
    Ok(result)
}
```

### Service Call Pattern
```rust
pub async fn handler(
    Json(req): Json<Request>,
    services: Extension<Arc<AppServices>>,
) -> AppResult<(StatusCode, Json<Response>)> {
    let result = services.service.operation(req).await?;
    Ok((StatusCode::CREATED, Json(result)))
}
```

### Repository Pattern
```rust
#[async_trait]
impl YourRepository for PgYourRepository {
    async fn create(&self, item: &Item) -> AppResult<Uuid> {
        sqlx::query_scalar("INSERT ... RETURNING id")
            .bind(&item.field)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }
}
```

## When to Ask Questions

Ask for clarification when:
- Architecture decisions affect multiple layers
- New entity/domain concept needed
- Complex business logic needs definition
- Integration with FSRS or AI checking required
- Database schema changes needed
- Deployment or infrastructure changes

## Performance Considerations

- Use connection pooling (via `PgPool`)
- Implement caching for frequently accessed data
- Consider pagination for large result sets
- Use indexes on frequently queried fields
- Monitor async task spawning

## Security Considerations

- Validate all user input in presentation layer
- Use parameterized queries (sqlx handles this)
- Implement authentication middleware
- Add authorization checks in services
- Log security events
- Sanitize error messages to clients

## Next Steps & TODOs

- [ ] Implement database migrations
- [ ] Add authentication/authorization
- [ ] Integrate FSRS algorithm for review scheduling
- [ ] Add AI-based answer checking service
- [ ] Implement event handlers for DDD events
- [ ] Add OpenAPI documentation
- [ ] Create comprehensive integration tests
- [ ] Set up CI/CD pipeline
- [ ] Implement caching strategies
- [ ] Add monitoring and observability

---

**Last Updated**: February 2026
**Architecture**: Hexagonal Architecture with SOLID principles
**Status**: MVP Foundation Ready

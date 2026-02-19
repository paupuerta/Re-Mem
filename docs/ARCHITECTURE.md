# ReMem Architecture Documentation

## Overview

ReMem is a language-learning application backend built with Rust, using **Hexagonal Architecture** (Ports & Adapters pattern) combined with Domain-Driven Design principles and an event-driven approach.

## Why Hexagonal Architecture?

Hexagonal Architecture separates concerns into distinct layers, making the codebase:
- **Testable**: Business logic is isolated from infrastructure
- **Maintainable**: Clear separation of concerns
- **Scalable**: Easy to add new adapters (APIs, databases, services)
- **DDD-Ready**: Foundation for Domain-Driven Design migration

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         PRESENTATION LAYER                       │
│                     (REST API via Axum + OpenAPI)               │
│  /users  /cards  /reviews  ... (HTTP Handlers)                 │
└──────────────────┬──────────────────────────────────────────────┘
                   │
         Uses Application Services
                   │
┌──────────────────▼──────────────────────────────────────────────┐
│                    APPLICATION LAYER                            │
│              (Use Cases & Application Services)                 │
│  UserService  CardService  ReviewService                        │
│  - Orchestrate domain logic                                      │
│  - Handle business workflows                                     │
└──────────────────┬──────────────────────────────────────────────┘
                   │
      Uses Repository Interfaces & Events
                   │
        ┌──────────▼──────────┐
        │                     │
┌───────▼────────┐    ┌──────▼──────────┐
│  DOMAIN LAYER  │    │ SHARED LAYER    │
│                │    │                 │
│ - Entities     │    │ - Event Bus     │
│ - Value Objs   │    │ - Error Types   │
│ - Repositories │    │ - Traits        │
│   (Interfaces) │    │                 │
└───────┬────────┘    └─────────────────┘
        │
    Implemented by
        │
┌───────▼──────────────────────────────────────────────────────────┐
│               INFRASTRUCTURE LAYER                               │
│              (Database & External Services)                      │
│  PgUserRepository  PgCardRepository  PgReviewRepository         │
│  Database Connection Pool  External API Clients                 │
└───────────────────────────────────────────────────────────────────┘
```

## Layer Details

### 1. Domain Layer (`src/domain/`)

The **heart of the application** containing pure business logic.

**Key Concepts:**
- **Entities**: Objects with unique identity (User, Card, Review)
- **Value Objects**: Immutable objects representing values (Email, Grade)
- **Aggregates**: Groups of entities that work together
- **Repository Interfaces**: Contracts for data persistence (not implementations!)

**Key Files:**
- `entities.rs` - Core domain models
- `value_objects.rs` - Value objects with validation
- `repositories.rs` - Repository trait definitions

**Rules:**
- No external dependencies (except serialization)
- Contains business rules and validation
- Independent of frameworks and databases

### 2. Application Layer (`src/application/`)

**Orchestrates** domain logic for specific use cases.

**Key Concepts:**
- **Application Services**: Coordinate domain entities to fulfill use cases
- **DTOs**: Data Transfer Objects for request/response
- **Use Cases**: High-level application operations

**Key Services:**
- `UserService` - User management
- `CardService` - Flashcard operations  
- `ReviewService` - Study sessions and FSRS integration

**Key Files:**
- `services.rs` - Application service implementations
- `dtos.rs` - Request/Response data structures
- `use_cases.rs` - Use case definitions

**Rules:**
- Depends on domain layer
- No HTTP/database details
- Converts between domain and external representations

### 3. Presentation Layer (`src/presentation/`)

**Exposes the API** to the outside world.

**Key Concepts:**
- **Handlers**: Process HTTP requests
- **Routing**: Maps URLs to handlers
- **Request/Response Mapping**: Converts HTTP data to DTOs and back

**Endpoints:**
- `POST /users` - Create user
- `GET /users/:id` - Get user
- `POST /users/:id/cards` - Create card
- `GET /users/:id/cards` - List user's cards
- `POST /users/:id/cards/:card_id/reviews` - Submit review

**Key Files:**
- `handlers.rs` - HTTP handler functions
- `router.rs` - Route definitions and dependency injection

**Rules:**
- Thin layer (minimal logic)
- Uses application services
- Maps HTTP to domain representations

### 4. Infrastructure Layer (`src/infrastructure/`)

**Implements** the contracts defined in the domain layer.

**Key Concepts:**
- **Repository Implementations**: Concrete database access
- **Database Configuration**: Connection pooling, migrations
- **External Service Clients**: AI checking, FSRS integration

**Key Files:**
- `repositories.rs` - PostgreSQL repository implementations
- `database.rs` - Database setup and migrations

**Rules:**
- Implements domain repository traits
- Contains all database-specific SQL
- Isolated from business logic

### 5. Shared Layer (`src/shared/`)

**Cross-cutting concerns** used by all layers.

**Key Concepts:**
- **Event Bus**: In-memory event system (future persistence)
- **Error Handling**: Unified error types and HTTP responses
- **Logging**: Tracing and observability

**Key Files:**
- `error.rs` - `AppError` type and error handling
- `event_bus.rs` - Event publishing and handling system

**Rules:**
- No business logic
- Reusable by all layers
- Infrastructure-focused

## Data Flow Example: Creating a Card

```
HTTP Request
    │
    ▼
┌───────────────────────────────────────────┐
│ Handler (Presentation Layer)              │
│ POST /users/:id/cards                     │
│ - Parse request body                      │
│ - Extract user_id from path               │
│ - Call service                            │
└───────────────────┬───────────────────────┘
                    │ CreateCardRequest DTO
                    ▼
┌───────────────────────────────────────────┐
│ CardService (Application Layer)           │
│ create_card(user_id, request)             │
│ - Validate input                          │
│ - Create Card entity                      │
│ - Call repository.create()                │
│ - Publish CardCreatedEvent                │
└───────────────────┬───────────────────────┘
                    │ Card entity
                    ▼
┌───────────────────────────────────────────┐
│ PgCardRepository (Infrastructure Layer)   │
│ create(card)                              │
│ - Map Card to SQL INSERT                  │
│ - Execute query                           │
│ - Return UUID                             │
└───────────────────┬───────────────────────┘
                    │ UUID
                    ▼
HTTP Response (201 Created)
CardDto { id, user_id, question, answer }
```

## Event Bus for DDD Migration

The `EventBus` system is designed to evolve into full Domain-Driven Design:

**Current State (MVP):**
- In-memory event distribution
- DomainEvent trait for event modeling
- EventHandler trait for subscribers

**Future State (DDD):**
- Event persistence (Event Store)
- Event replay capabilities
- Sagas for multi-domain operations
- Complete event sourcing

**Example:**
```rust
// Publish event when card is created
let card = Card::new(user_id, question, answer);
event_bus.publish(CardCreatedEvent { 
    card_id: card.id,
    user_id,
    timestamp: Utc::now(),
}).await?;
```

## Dependency Injection Pattern

The project uses **constructor injection** with `Arc` for shared ownership:

```rust
// In main.rs
let user_repo = Arc::new(PgUserRepository::new(db_pool.clone()));
let user_service = Arc::new(UserService::new(user_repo));

// Services are passed via Axum Extension
let app = create_router(Arc::new(AppServices { 
    user_service,
    // ...
}));
```

This approach:
- Makes dependencies explicit
- Enables testing with mock implementations
- Avoids global state
- Supports SOLID principles

## SOLID Principles Applied

### Single Responsibility
- `UserRepository` only handles user persistence
- `UserService` only orchestrates user operations
- Each handler manages one endpoint

### Open/Closed Principle
- Repository trait allows new implementations without changing domain
- Can add `MongoUserRepository` without touching existing code

### Liskov Substitution
- Any implementation of `UserRepository` can be used in place of another
- Enables testing with `MockUserRepository`

### Interface Segregation
- Narrow, focused traits (don't depend on unnecessary methods)
- `CardRepository` doesn't require user methods

### Dependency Inversion
- Application layer depends on repository **traits**, not concrete types
- High-level modules don't depend on low-level modules

## Testing Strategy

### Unit Tests
Located near implementation:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_email_validation() {
        assert!(Email::new("valid@test.com".into()).is_ok());
        assert!(Email::new("invalid".into()).is_err());
    }
}
```

### Integration Tests
Located in `tests/` directory:
```rust
#[tokio::test]
async fn test_create_user_workflow() {
    // Full workflow from handler to database
}
```

### Test Doubles
- Use trait implementations for mocking
- In-memory repositories for testing

## Technology Stack

| Layer | Technology |
|-------|-----------|
| Runtime | Tokio (async) |
| HTTP | Axum + Tower |
| Database | PostgreSQL + SQLx |
| Serialization | Serde + serde_json |
| Validation | Value objects + Result types |
| Error Handling | Thiserror + custom AppError |
| Logging | Tracing + tracing-subscriber |
| FSRS | fsrs-rs crate |
| ID Generation | UUID v4 |

## Database Schema (To Be Implemented)

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR NOT NULL UNIQUE,
    name VARCHAR NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE cards (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE reviews (
    id UUID PRIMARY KEY,
    card_id UUID NOT NULL REFERENCES cards(id),
    user_id UUID NOT NULL REFERENCES users(id),
    grade INT NOT NULL CHECK (grade >= 0 AND grade <= 5),
    created_at TIMESTAMP DEFAULT NOW()
);
```

## Configuration

Environment variables (see `.env.example`):
```bash
DATABASE_URL=postgres://user:password@localhost:5432/re_mem
RUST_LOG=info
RUST_LOG=re_mem=debug
```

## Running Locally

```bash
# Install dependencies
cargo build

# Run migrations
cargo sqlx migrate run

# Start development server
cargo run

# Run tests
cargo test

# Format and lint
cargo fmt && cargo clippy
```

## Deployment

### Docker
```bash
docker build -t re-mem:latest .
docker run -e DATABASE_URL=... re-mem:latest
```

### Docker Compose (Development)
```bash
docker-compose up
```

### Kubernetes
Manifests in `k8s/` directory:
```bash
kubectl apply -f k8s/
```

## Future Enhancements

1. **Event Sourcing**: Persist events to database
2. **CQRS**: Separate read and write models
3. **Authentication**: JWT token-based auth
4. **Authorization**: Role-based access control
5. **API Documentation**: OpenAPI/Swagger integration
6. **Caching**: Redis for frequently accessed data
7. **Monitoring**: Prometheus metrics
8. **Async Jobs**: Background task processing
9. **WebSockets**: Real-time updates
10. **Microservices**: Separate AI checking service

---

**Architecture Decision Date**: February 2026  
**Review Date**: Q2 2026  
**Status**: MVP Foundation

# Database Documentation

## Overview

ReMem uses PostgreSQL as the primary data store with SQLx for type-safe database access.

## Database Setup

### Local Development

```bash
# Create database
createdb re_mem

# Run migrations
cargo sqlx migrate run

# Verify
psql re_mem -c "SELECT 1"
```

### Docker Compose

The `docker-compose.yml` includes PostgreSQL:

```bash
docker-compose up postgres
```

Access:

- Host: localhost
- Port: 5432
- User: re_mem
- Password: password
- Database: re_mem

## Schema

### Users Table

Stores user information for the language learning application.

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_users_email (email)
);
```

**Fields:**

- `id`: Unique identifier (UUID)
- `email`: User email (unique)
- `name`: User's full name
- `created_at`: Account creation timestamp
- `updated_at`: Last modification timestamp

### Cards Table

Stores flashcards with questions and answers.

```sql
CREATE TABLE cards (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    answer TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_cards_user_id (user_id)
);
```

**Fields:**

- `id`: Unique card identifier
- `user_id`: Owner of the card (foreign key)
- `question`: The question/prompt
- `answer`: The correct answer
- `created_at`: Card creation timestamp
- `updated_at`: Last modification timestamp

### Reviews Table

Stores study session records for FSRS algorithm.

```sql
CREATE TABLE reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    card_id UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    grade INT NOT NULL CHECK (grade >= 0 AND grade <= 5),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    INDEX idx_reviews_card_id (card_id),
    INDEX idx_reviews_user_id (user_id)
);
```

**Fields:**

- `id`: Unique review identifier
- `card_id`: Reviewed card (foreign key)
- `user_id`: User who submitted review
- `grade`: FSRS grade (0-5)
- `created_at`: Review submission timestamp

**Grade Scale (FSRS):**

- 0: Again (forgot the answer)
- 1: Hard (difficult to recall)
- 2: Good (correct recall)
- 3: Easy (easy to recall)
- 4: Very Easy (very easy recall)
- 5: Perfect (instantaneous correct)

### FSRS State Table (Future)

For tracking FSRS algorithm state per card:

```sql
CREATE TABLE card_fsrs_states (
    id UUID PRIMARY KEY,
    card_id UUID NOT NULL UNIQUE REFERENCES cards(id),
    stability FLOAT8 NOT NULL DEFAULT 1.0,
    difficulty FLOAT8 NOT NULL DEFAULT 5.0,
    last_review TIMESTAMP WITH TIME ZONE,
    next_review TIMESTAMP WITH TIME ZONE,
    reps INT NOT NULL DEFAULT 0,
    lapses INT NOT NULL DEFAULT 0,
    state VARCHAR(50),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

## Migrations

Migrations are managed using `sqlx migrate` command.

### Create Migration

```bash
# Create a new migration
sqlx migrate add -r <migration_name>

# Example
sqlx migrate add -r create_users_table
```

This creates two files:

- `<timestamp>_<name>.up.sql` - Forward migration
- `<timestamp>_<name>.down.sql` - Rollback migration

### Up Migration Example

#### migrations/20260219000001_create_users_table.up.sql

```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_email ON users(email);
```

### Down Migration Example

#### migrations/20260219000001_create_users_table.down.sql

```sql
DROP INDEX IF EXISTS idx_users_email;
DROP TABLE IF EXISTS users CASCADE;
```

### Run Migrations

```bash
# Run all pending migrations
cargo sqlx migrate run

# Verify
cargo sqlx database info
```

## Performance Considerations

### Indexing Strategy

Current indexes:

1. `users.email` - Used in find_by_email queries
2. `cards.user_id` - Used for filtering user's cards
3. `reviews.card_id` - Used for finding reviews of a card
4. `reviews.user_id` - Used for finding user's reviews

### Query Optimization

SQLx compilation-time verification ensures:

- SQL syntax is valid
- Column names match schema
- Type mismatches are caught early

### Connection Pooling

Default pool configuration:

```rust
PgPoolOptions::new()
    .max_connections(5)  // Adjust based on load
    .connect(&database_url)
    .await?
```

### Future Optimizations

- Add caching layer (Redis)
- Implement query result caching
- Batch review queries
- Archive old reviews
- Partition large tables by date

## Backup and Recovery

### Backup

```bash
# Using docker-compose
docker-compose exec postgres pg_dump -U re_mem re_mem > backup.sql

# Using psql
pg_dump -U re_mem re_mem > backup.sql
```

### Restore

```bash
# Using docker-compose
docker-compose exec -T postgres psql -U re_mem re_mem < backup.sql

# Using psql
psql -U re_mem re_mem < backup.sql
```

## Development Tips

### Connect to Database

```bash
# Using psql
psql postgres://re_mem:password@localhost:5432/re_mem

# Using docker-compose
docker-compose exec postgres psql -U re_mem re_mem
```

### View Tables

```sql
\dt  -- List tables
\d+ cards  -- Describe table details
\i migrations/...sql  -- Execute migration file
```

### Reset Database (Development Only)

```bash
# Using docker-compose
docker-compose down -v
docker-compose up postgres

# Manually
dropdb re_mem
createdb re_mem
cargo sqlx migrate run
```

## Monitoring

### Connection Monitoring

```sql
SELECT usename, count(*) 
FROM pg_stat_activity 
GROUP BY usename;
```

### Slow Queries

Enable logging in PostgreSQL config:

```sql
SET log_min_duration_statement = 1000;  -- Log queries > 1 second
```

## Testing

### Test Database Setup

```bash
# Create test database
createdb re_mem_test

# Run migrations on test database
DATABASE_URL=postgres://user:password@localhost/re_mem_test \
cargo sqlx migrate run
```

### In-Memory Testing

Use SQLx's `setup` feature for test fixtures:

```rust
#[sqlx::test]
async fn test_create_user(pool: PgPool) {
    // Test code here
}
```

---

For implementation details, see `src/infrastructure/database.rs` and `src/infrastructure/repositories.rs`.

# Development Setup Guide

## Prerequisites

### Required
- Rust 1.70+ ([Install](https://rustup.rs/))
- PostgreSQL 13+ ([Install](https://www.postgresql.org/download/))
- Docker & Docker Compose (for containerized development)
- Git

### Optional
- VS Code with rust-analyzer extension
- PostgreSQL clients (psql, pgAdmin)
- Postman or similar API testing tool

## Quick Start

### 1. Clone Repository

```bash
cd ~/Code/ReMem
git clone <repo-url> re-mem
cd re-mem
```

### 2. Environment Setup

```bash
# Copy example environment file
cp .env.example .env

# Edit with your local settings
# Required:
# DATABASE_URL=postgres://user:password@localhost:5432/re_mem
# RUST_LOG=info
```

### 3. Database Setup

#### Option A: Local PostgreSQL

```bash
# Create database
createdb re_mem

# Create user (optional)
createuser -P re_mem  # Password: re_mem

# Grant privileges
psql -U postgres -d re_mem -c "GRANT ALL ON SCHEMA public TO re_mem;"
```

#### Option B: Docker Compose (Recommended)

```bash
docker-compose up -d postgres
# Waits ~5 seconds for database to be ready
sleep 5
```

### 4. Run Migrations

```bash
# Requires cargo-sqlx-cli
# cargo install sqlx-cli --no-default-features --features postgres

# Run migrations
cargo sqlx migrate run

# Verify
cargo sqlx database create
```

### 5. Build and Run

```bash
# Build the project
cargo build

# Run development server
cargo run

# Server starts on http://localhost:3000
# Health check: curl http://localhost:3000/health
```

## Development Workflow

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test --lib domain::value_objects

# Run integration tests
cargo test --test '*'

# Run with output
cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy linter
cargo clippy -- -D warnings

# Check formatting
cargo fmt -- --check

# Full quality check
cargo fmt && cargo clippy && cargo test
```

### Debugging

```bash
# Run with debug output
RUST_LOG=debug cargo run

# Set specific module debug
RUST_LOG=re_mem=debug,axum=info cargo run

# Use rust-gdb
rust-gdb ./target/debug/re_mem
```

### Hot Reload

Install and use `cargo-watch`:

```bash
cargo install cargo-watch

# Watch and rebuild on file changes
cargo watch -x run

# Watch and run tests
cargo watch -x test
```

## IDE Setup

### VS Code

#### Extensions
- `rust-analyzer` - Language support
- `Better TOML` - Cargo.toml support
- `SQLTools` - Database tools
- `REST Client` - API testing

#### Settings (.vscode/settings.json)
```json
{
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer",
        "editor.formatOnSave": true
    },
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.inlayHints.enable": true
}
```

#### Debugging
1. Install `CodeLLDB` extension
2. Create `.vscode/launch.json`:
```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug re_mem",
            "cargo": {
                "args": [
                    "build",
                    "--bin=re_mem",
                    "--package=re_mem"
                ],
                "filter": {
                    "name": "re_mem",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

## Testing API Locally

### Using curl

```bash
# Health check
curl http://localhost:3000/health

# Create user
curl -X POST http://localhost:3000/users \
  -H "Content-Type: application/json" \
  -d '{"email": "test@example.com", "name": "Test User"}'

# Get user
curl http://localhost:3000/users/550e8400-e29b-41d4-a716-446655440000
```

### Using REST Client Extension

Create `test.http`:
```http
@baseUrl = http://localhost:3000

### Health Check
GET {{baseUrl}}/health

### Create User
POST {{baseUrl}}/users
Content-Type: application/json

{
  "email": "test@example.com",
  "name": "Test User"
}

### Get User
GET {{baseUrl}}/users/550e8400-e29b-41d4-a716-446655440000
```

Then click "Send Request" in VS Code.

### Using Postman

1. Import collection from `docs/postman-collection.json`
2. Set environment variables:
   - `baseUrl`: http://localhost:3000
   - `userId`: (set after creating user)
3. Run requests

## Database Management

### View Tables

```bash
# Connect to database
psql postgres://re_mem:password@localhost:5432/re_mem

# List tables
\dt

# Describe table
\d+ cards

# Run query
SELECT * FROM users;
```

### Reset Database

```bash
# Drop all tables (development only!)
cargo sqlx database drop -y

# Recreate and migrate
cargo sqlx database create
cargo sqlx migrate run
```

### Export/Import

```bash
# Export
pg_dump postgres://re_mem:password@localhost:5432/re_mem > backup.sql

# Import
psql postgres://re_mem:password@localhost:5432/re_mem < backup.sql
```

## Docker Development

### With Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Rebuild images
docker-compose build --no-cache
```

### Build Docker Image

```bash
docker build -t re-mem:latest .

docker run -e DATABASE_URL=postgres://... \
  -p 3000:3000 \
  re-mem:latest
```

## Kubernetes Development

### Setup local cluster

```bash
# Using minikube
minikube start

# Using kind
kind create cluster

# Using docker-desktop built-in kubernetes
# (Enable in Docker Desktop settings)
```

### Deploy to local cluster

```bash
# Build image for cluster
docker build -t re-mem:local .

# Apply manifests
kubectl apply -f k8s/

# Check status
kubectl get pods
kubectl get services

# Port forward for testing
kubectl port-forward svc/re-mem-svc 3000:3000

# View logs
kubectl logs -f deployment/re-mem
```

## Performance Profiling

### Using flamegraph

```bash
cargo install flamegraph

cargo flamegraph --bin re_mem

# Generates flamegraph.svg
open flamegraph.svg
```

### Using perf

```bash
# On Linux
cargo build --release
perf record --call-graph=dwarf ./target/release/re_mem
perf report
```

## Troubleshooting

### Compilation Issues

```bash
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update
cargo check
```

### Database Connection

```bash
# Test connection
psql postgres://re_mem:password@localhost:5432/re_mem -c "SELECT 1"

# Check environment variable
echo $DATABASE_URL

# Verify credentials
# User: re_mem
# Password: password (default in .env)
# Host: localhost
# Port: 5432
# Database: re_mem
```

### Port Already in Use

```bash
# Change port in main.rs or set via environment
PORT=3001 cargo run

# Or kill existing process
kill -9 $(lsof -t -i:3000)
```

## Useful Commands

```bash
# Build in release mode
cargo build --release

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --open

# Check for outdated dependencies
cargo outdated

# Audit for vulnerabilities
cargo audit

# Generate lock file
cargo lock
```

## Contributing

See [CONTRIBUTING.md](/CONTRIBUTING.md) for guidelines.

---

**Last Updated**: February 2026  
**Rust Version**: 1.70+  
**Status**: MVP Development

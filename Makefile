.PHONY: help build run test fmt lint clean docker docker-build docker-compose k8s-deploy

help:
	@echo "ReMem - Language Learning Backend"
	@echo ""
	@echo "Available targets:"
	@echo "  build            - Build the project"
	@echo "  run              - Run development server"
	@echo "  test             - Run all tests"
	@echo "  test-unit        - Run unit tests only"
	@echo "  test-integration - Run integration tests"
	@echo "  fmt              - Format code"
	@echo "  lint             - Run clippy linter"
	@echo "  check            - Format + lint + test"
	@echo "  clean            - Clean build artifacts"
	@echo "  docker-build     - Build Docker image"
	@echo "  docker-up        - Start Docker Compose services"
	@echo "  docker-up-build  - Build backend and start Docker Compose services"
	@echo "  docker-down      - Stop Docker Compose services"
	@echo "  dev              - Start dev environment with hot-reload"
	@echo "  dev-down         - Stop dev environment"
	@echo "  dev-logs         - Follow dev backend logs"
	@echo "  db-create        - Create database"
	@echo "  db-migrate       - Run database migrations"
	@echo "  db-reset         - Reset database (dev only)"
	@echo "  docs             - Generate and open documentation"
	@echo "  k8s-deploy       - Deploy to Kubernetes"
	@echo "  k8s-status       - Check Kubernetes deployment status"

build:
	cargo build

run:
	cargo run

test:
	cargo test --all

test-unit:
	cargo test --lib

test-integration:
	cargo test --test '*'

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

check: fmt lint test
	@echo "? All checks passed!"

clean:
	cargo clean

docker-build:
	docker build -t re-mem:latest -f .docker/Dockerfile .

docker-up-build:
	docker-compose up -d --build backend
	@echo "? Docker services started"
	@echo "  API: http://localhost:3000"
	@echo "  pgAdmin: http://localhost:5050"
	@echo "  Database: postgres://re_mem:password@localhost:5432/re_mem"

docker-up:
	docker-compose up -d
	@echo "? Docker services started"
	@echo "  API: http://localhost:3000"
	@echo "  pgAdmin: http://localhost:5050"
	@echo "  Database: postgres://re_mem:password@localhost:5432/re_mem"

docker-down:
	docker-compose down

dev:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml up -d --build backend
	@echo "  Dev server starting with hot-reload"
	@echo "  API: http://localhost:3000"
	@echo "  pgAdmin: http://localhost:5050"
	@echo "  Logs: make dev-logs"

dev-down:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml down

dev-logs:
	docker-compose -f docker-compose.yml -f docker-compose.dev.yml logs -f backend

docker-logs:
	docker-compose logs -f

db-create:
	createdb re_mem 2>/dev/null || echo "Database already exists"

db-migrate:
	cargo sqlx migrate run

db-reset:
	@echo "??  Resetting database..."
	cargo sqlx database drop -y
	cargo sqlx database create
	cargo sqlx migrate run
	@echo "? Database reset complete"

docs:
	cargo doc --open

k8s-deploy:
	kubectl apply -f k8s/
	@echo "? Kubernetes deployment applied"
	@echo "  Namespace: re-mem"
	@echo "  Port forward: kubectl port-forward svc/re-mem-svc 3000:3000 -n re-mem"

k8s-status:
	kubectl get all -n re-mem

k8s-logs:
	kubectl logs -f deployment/re-mem -n re-mem

k8s-delete:
	kubectl delete -f k8s/

watch:
	cargo watch -x run

release:
	cargo build --release

setup-dev: docker-up db-migrate build
	@echo "? Development environment setup complete"
	@echo "  Run 'make run' to start the server"

.DEFAULT_GOAL := help

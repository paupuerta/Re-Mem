//! ReMem - Language Learning Backend
//!
//! A Rust backend for a language-learning MVP using FSRS (Free Spaced Repetition Scheduler)
//! and AI-based answer checking with Hexagonal Architecture.
//!
//! # Architecture
//!
//! The project follows Hexagonal Architecture (Ports & Adapters) with the following layers:
//! - **domain**: Core business logic and entities
//! - **application**: Use cases and application services
//! - **infrastructure**: Database, external APIs, and storage implementations
//! - **presentation**: REST API endpoints
//! - **shared**: Event bus, error handling, and cross-cutting concerns

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod presentation;
pub mod shared;

pub use shared::{AppError, AppResult};

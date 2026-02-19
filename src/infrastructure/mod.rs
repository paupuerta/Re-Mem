//! Infrastructure layer - Database, external APIs, and implementations
//! 
//! This layer contains:
//! - Repository implementations for database access
//! - External API clients
//! - Configuration and dependency injection
//! 
//! SOLID:
//! - D: Depends on abstractions (domain repository traits)
//! - O: Extensions without modifying domain

pub mod database;
pub mod repositories;

pub use repositories::*;

//! Application layer - Use cases and application services
//!
//! This layer contains:
//! - Use Cases: High-level application operations
//! - Application Services: Orchestrate domain entities and repositories
//! - DTOs: Data Transfer Objects for request/response
//!
//! Follows:
//! - SOLID principles (especially S and I)
//! - KISS: Keep solutions simple and straightforward
//! - YAGNI: You Aren't Gonna Need It

pub mod dtos;
pub mod services;
pub mod use_cases;

pub use dtos::*;
pub use services::*;
pub use use_cases::*;

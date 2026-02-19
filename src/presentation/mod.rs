//! Presentation layer - REST API endpoints
//! 
//! This layer contains:
//! - HTTP handlers
//! - Request/Response serialization
//! - OpenAPI documentation
//! - Route configuration
//! 
//! SOLID:
//! - S: Each handler focuses on one endpoint
//! - I: Handlers use specific DTOs

pub mod handlers;
pub mod router;

pub use router::create_router;

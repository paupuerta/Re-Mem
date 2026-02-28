//! Shared module containing cross-cutting concerns
//! Including event bus, error handling, and utilities

pub mod error;
pub mod event_bus;
pub mod jwt;

pub use error::{AppError, AppResult};
pub use event_bus::{DomainEvent, EventBus, EventHandler};

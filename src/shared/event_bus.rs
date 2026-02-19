use std::any::Any;
use uuid::Uuid;

/// Domain event trait for implementing domain-driven design
/// These events represent meaningful business occurrences that happened in the system
pub trait DomainEvent: Send + Sync {
    /// Unique event ID for idempotency and tracking
    fn event_id(&self) -> Uuid;
    
    /// Timestamp when the event occurred
    fn event_timestamp(&self) -> chrono::DateTime<chrono::Utc>;
    
    /// Event name for event routing and logging
    fn event_name(&self) -> &'static str;
    
    /// Used for type-erased storage
    fn as_any(&self) -> &dyn Any;
}

/// Event handler trait for processing domain events
#[async_trait::async_trait]
pub trait EventHandler<E: DomainEvent + ?Sized>: Send + Sync {
    async fn handle(&self, event: &E) -> crate::AppResult<()>;
}

/// In-memory event bus for handling domain events
/// This will evolve into a proper event sourcing system for DDD migration
pub struct EventBus {
    // Placeholder for event storage and handlers
    // TODO: Implement with actual subscriber registry and event store
}

impl EventBus {
    pub fn new() -> Self {
        Self {}
    }

    /// Publish a domain event
    pub async fn publish<E: DomainEvent + 'static>(&self, _event: E) -> crate::AppResult<()> {
        // TODO: Route event to registered handlers
        tracing::info!("Event published: {}", _event.event_name());
        Ok(())
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

use uuid::Uuid;

/// Simple domain events enum for basic event-driven architecture
#[derive(Debug, Clone)]
pub enum DomainEvent {
    CardReviewed {
        card_id: Uuid,
        user_id: Uuid,
        score: f32,
        rating: i32,
    },
    CardCreated {
        card_id: Uuid,
        user_id: Uuid,
    },
}

/// Event handler trait for processing domain events
#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: DomainEvent) -> crate::AppResult<()>;
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
    pub async fn publish(&self, event: DomainEvent) {
        // TODO: Route event to registered handlers
        tracing::info!("Event published: {:?}", event);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

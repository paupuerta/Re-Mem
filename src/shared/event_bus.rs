use std::sync::Arc;
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
        deck_id: Option<Uuid>,
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
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Register an event handler
    pub fn register_handler(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }

    /// Publish a domain event to all registered handlers
    pub async fn publish(&self, event: DomainEvent) {
        tracing::info!("Event published: {:?}", event);
        for handler in &self.handlers {
            if let Err(e) = handler.handle(event.clone()).await {
                tracing::error!("Event handler error: {:?}", e);
            }
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

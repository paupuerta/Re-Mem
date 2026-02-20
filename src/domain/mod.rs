//! Domain layer - Core business logic and entities
//! 
//! This layer contains:
//! - Entities: Objects with unique identities
//! - Value Objects: Immutable objects without unique identities
//! - Aggregates: Clusters of entities and value objects
//! - Repositories: Abstract interfaces for persistence
//! - Domain Events: Business-meaningful events
//! 
//! Following SOLID principles:
//! - S: Each module has a single responsibility
//! - O: Open for extension, closed for modification
//! - L: Liskov Substitution Principle via traits
//! - I: Interface Segregation via focused traits
//! - D: Dependency Inversion via repository interfaces

pub mod entities;
pub mod repositories;
pub mod value_objects;
pub mod ports;

pub use entities::*;
pub use repositories::*;
pub use value_objects::*;
pub use ports::*;

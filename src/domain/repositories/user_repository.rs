use crate::{domain::entities::User, AppResult};
use uuid::Uuid;

/// Repository interface for User domain
/// SOLID: Interface Segregation and Dependency Inversion
#[async_trait::async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> AppResult<Uuid>;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>>;
    async fn find_by_email(&self, email: &str) -> AppResult<Option<User>>;
    async fn update(&self, user: &User) -> AppResult<()>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;
}

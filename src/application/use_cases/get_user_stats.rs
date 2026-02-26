use std::sync::Arc;
use uuid::Uuid;

use crate::{
    application::dtos::UserStatsDto,
    domain::repositories::UserStatsRepository,
    AppResult,
};

/// Use case for retrieving user statistics
pub struct GetUserStatsUseCase {
    user_stats_repository: Arc<dyn UserStatsRepository>,
}

impl GetUserStatsUseCase {
    pub fn new(user_stats_repository: Arc<dyn UserStatsRepository>) -> Self {
        Self {
            user_stats_repository,
        }
    }

    pub async fn execute(&self, user_id: Uuid) -> AppResult<UserStatsDto> {
        let stats = self.user_stats_repository.get_or_create(user_id).await?;

        Ok(UserStatsDto {
            user_id: stats.user_id,
            total_reviews: stats.total_reviews,
            correct_reviews: stats.correct_reviews,
            days_studied: stats.days_studied,
            accuracy_percentage: stats.accuracy_percentage(),
            last_active_date: stats.last_active_date.map(|d| d.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::UserStats;
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct MockUserStatsRepository {
        stats: Mutex<Option<UserStats>>,
    }

    impl MockUserStatsRepository {
        fn new() -> Self {
            Self {
                stats: Mutex::new(None),
            }
        }

        fn with_stats(stats: UserStats) -> Self {
            Self {
                stats: Mutex::new(Some(stats)),
            }
        }
    }

    #[async_trait]
    impl UserStatsRepository for MockUserStatsRepository {
        async fn get_or_create(&self, user_id: Uuid) -> AppResult<UserStats> {
            let stats = self.stats.lock().unwrap();
            Ok(stats.clone().unwrap_or_else(|| UserStats::new(user_id)))
        }

        async fn update_after_review(
            &self,
            _user_id: Uuid,
            _is_correct: bool,
            _review_date: chrono::NaiveDate,
        ) -> AppResult<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_get_user_stats_new_user() {
        let user_id = Uuid::new_v4();
        let repo = Arc::new(MockUserStatsRepository::new());
        let use_case = GetUserStatsUseCase::new(repo);

        let result = use_case.execute(user_id).await.unwrap();

        assert_eq!(result.user_id, user_id);
        assert_eq!(result.total_reviews, 0);
        assert_eq!(result.correct_reviews, 0);
        assert_eq!(result.days_studied, 0);
        assert_eq!(result.accuracy_percentage, 0.0);
    }

    #[tokio::test]
    async fn test_get_user_stats_with_data() {
        let user_id = Uuid::new_v4();
        let mut stats = UserStats::new(user_id);
        stats.total_reviews = 100;
        stats.correct_reviews = 80;
        stats.days_studied = 15;

        let repo = Arc::new(MockUserStatsRepository::with_stats(stats));
        let use_case = GetUserStatsUseCase::new(repo);

        let result = use_case.execute(user_id).await.unwrap();

        assert_eq!(result.total_reviews, 100);
        assert_eq!(result.correct_reviews, 80);
        assert_eq!(result.days_studied, 15);
        assert_eq!(result.accuracy_percentage, 80.0);
    }
}

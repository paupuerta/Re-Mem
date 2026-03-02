use crate::{
    domain::{entities::Review, repositories::ReviewRepository},
    AppResult,
};
use std::sync::Arc;
use uuid::Uuid;

use super::super::dtos::{LegacyReviewCardRequest, ReviewDto};

/// Review service - handles review/study operations using FSRS
pub struct ReviewService {
    review_repo: Arc<dyn ReviewRepository>,
}

impl ReviewService {
    pub fn new(review_repo: Arc<dyn ReviewRepository>) -> Self {
        Self { review_repo }
    }

    pub async fn submit_review(
        &self,
        card_id: Uuid,
        user_id: Uuid,
        req: LegacyReviewCardRequest,
    ) -> AppResult<ReviewDto> {
        let review = Review::new(card_id, user_id, req.grade);
        let review_id = self.review_repo.create(&review).await?;

        Ok(ReviewDto {
            id: review_id,
            card_id: review.card_id,
            grade: review.grade,
        })
    }
}

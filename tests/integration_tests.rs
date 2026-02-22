//! Integration tests for ReMem

use re_mem::domain::entities::{Card, Deck, Review, User};

#[test]
fn test_user_creation() {
    let user = User::new("test@example.com".to_string(), "Test User".to_string());
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.name, "Test User");
    assert!(!user.id.is_nil());
}

#[test]
fn test_deck_creation() {
    let user_id = uuid::Uuid::new_v4();
    let deck = Deck::new(
        user_id,
        "Spanish Vocabulary".to_string(),
        Some("Basic Spanish words".to_string()),
    );
    assert_eq!(deck.name, "Spanish Vocabulary");
    assert_eq!(deck.description, Some("Basic Spanish words".to_string()));
    assert_eq!(deck.user_id, user_id);
    assert!(!deck.id.is_nil());
}

#[test]
fn test_deck_creation_without_description() {
    let user_id = uuid::Uuid::new_v4();
    let deck = Deck::new(user_id, "French Verbs".to_string(), None);
    assert_eq!(deck.name, "French Verbs");
    assert_eq!(deck.description, None);
    assert_eq!(deck.user_id, user_id);
}

#[test]
fn test_card_creation() {
    let user_id = uuid::Uuid::new_v4();
    let card = Card::new(user_id, "What is 2+2?".to_string(), "4".to_string());
    assert_eq!(card.question, "What is 2+2?");
    assert_eq!(card.answer, "4");
    assert_eq!(card.user_id, user_id);
    assert_eq!(card.deck_id, None);
    assert_eq!(card.answer_embedding, None);
}

#[test]
fn test_card_with_deck() {
    let user_id = uuid::Uuid::new_v4();
    let deck_id = uuid::Uuid::new_v4();
    let card = Card::new(user_id, "Test question".to_string(), "Test answer".to_string())
        .with_deck(deck_id);
    
    assert_eq!(card.deck_id, Some(deck_id));
    assert_eq!(card.user_id, user_id);
}

#[test]
fn test_card_with_embedding() {
    let user_id = uuid::Uuid::new_v4();
    let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let card = Card::new(user_id, "Test".to_string(), "Answer".to_string())
        .with_embedding(embedding.clone());
    
    assert_eq!(card.answer_embedding, Some(embedding));
}

#[test]
fn test_card_builder_pattern() {
    let user_id = uuid::Uuid::new_v4();
    let deck_id = uuid::Uuid::new_v4();
    let embedding = vec![0.1, 0.2, 0.3];
    
    let card = Card::new(user_id, "Q".to_string(), "A".to_string())
        .with_deck(deck_id)
        .with_embedding(embedding.clone());
    
    assert_eq!(card.deck_id, Some(deck_id));
    assert_eq!(card.answer_embedding, Some(embedding));
}

#[test]
fn test_review_creation() {
    let card_id = uuid::Uuid::new_v4();
    let user_id = uuid::Uuid::new_v4();
    let review = Review::new(card_id, user_id, 4);
    assert_eq!(review.grade, 4);
    assert_eq!(review.card_id, card_id);
    assert_eq!(review.user_id, user_id);
}

#[test]
fn test_value_object_email_validation() {
    use re_mem::domain::value_objects::Email;

    assert!(Email::new("valid@test.com".to_string()).is_ok());
    assert!(Email::new("invalid.email".to_string()).is_err());
}

#[test]
fn test_value_object_grade_validation() {
    use re_mem::domain::value_objects::Grade;

    assert!(Grade::new(0).is_ok());
    assert!(Grade::new(5).is_ok());
    assert!(Grade::new(6).is_err());
}

// TODO: Add async integration tests with database
// #[tokio::test]
// async fn test_user_repository_create() {
//     let pool = setup_test_db().await;
//     let repo = PgUserRepository::new(pool);
//     let user = User::new("test@example.com".into(), "Test".into());
//     let id = repo.create(&user).await.unwrap();
//     assert!(!id.is_nil());
// }

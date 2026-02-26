//! Use cases - High-level application workflows
//!
//! Each use case represents a single user action or interaction.
//! One file per use case following the Single Responsibility Principle.

pub mod create_card;
pub mod create_deck;
pub mod create_user;
pub mod delete_card;
pub mod delete_deck;
pub mod get_deck_stats;
pub mod get_decks;
pub mod get_user;
pub mod get_user_cards;
pub mod get_user_stats;
pub mod review_card;

pub use create_card::CreateCardUseCase;
pub use create_deck::CreateDeckUseCase;
pub use create_user::CreateUserUseCase;
pub use delete_card::DeleteCardUseCase;
pub use delete_deck::DeleteDeckUseCase;
pub use get_deck_stats::GetDeckStatsUseCase;
pub use get_decks::GetDecksUseCase;
pub use get_user::GetUserUseCase;
pub use get_user_cards::GetUserCardsUseCase;
pub use get_user_stats::GetUserStatsUseCase;
pub use review_card::{ReviewCardUseCase, ReviewResult};

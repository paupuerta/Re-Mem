//! Use cases - High-level application workflows
//! 
//! Each use case represents a single user action or interaction

pub struct CreateUserUseCase;
pub struct GetUserUseCase;
pub struct CreateCardUseCase;
pub struct GetUserCardsUseCase;
pub struct SubmitReviewUseCase;

// Use cases using services are implemented via handlers in the presentation layer
// This demonstrates KISS principle - we don't need separate use case classes,
// instead we compose services directly in handlers

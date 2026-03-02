use crate::domain::repositories::UserRepository;
use std::sync::Arc;

/// Auth service - handles registration and login
pub struct AuthService {
    pub register: Arc<super::super::use_cases::RegisterUserUseCase>,
    pub login: Arc<super::super::use_cases::LoginUserUseCase>,
}

impl AuthService {
    pub fn new(user_repo: Arc<dyn UserRepository>) -> Self {
        Self {
            register: Arc::new(super::super::use_cases::RegisterUserUseCase::new(
                user_repo.clone(),
            )),
            login: Arc::new(super::super::use_cases::LoginUserUseCase::new(user_repo)),
        }
    }
}

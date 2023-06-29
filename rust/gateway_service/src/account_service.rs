use std::fmt::Display;

use async_trait::async_trait;

use crate::username::Username;

#[derive(Debug, thiserror::Error)]
pub enum KeyValidationError {
    Invalid,
    Unknown(String),
}

impl Display for KeyValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyValidationError::Invalid => write!(f, "Invalid key"),
            KeyValidationError::Unknown(message) => write!(f, "Unknown error: {}", message),
        }
    }
}

#[async_trait]
pub trait AccountService {
    async fn verify_user_key(&self, username: &Username, key: &str) -> Result<(), KeyValidationError>;
}


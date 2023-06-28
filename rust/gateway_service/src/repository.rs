use std::fmt::Display;

use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    NotFound,
    Unknown(String),
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::NotFound => write!(f, "Not found"),
            RepositoryError::Unknown(message) => write!(f, "Unknown error: {}", message),
        }
    }
}

#[async_trait]
pub trait Repository<TEntity> {
    async fn read(&self, key: &str) -> Result<TEntity, RepositoryError>;
    async fn update(&self, entity: &TEntity) -> Result<(), RepositoryError>;
}

use async_trait::async_trait;

use crate::Username;

use super::{account_service::KeyValidationError, AccountService};

pub struct RemoteAccountService {
    url: String,
}

impl RemoteAccountService {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

#[async_trait]
impl AccountService for RemoteAccountService {
    async fn verify_user_key(&self, user: &Username, key: &str) -> Result<(), KeyValidationError> {
        let url = format!("{}/verify/{}/{}", self.url, user, key);

        let response = reqwest::get(&url)
            .await
            .map_err(|e| KeyValidationError::Unknown(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(KeyValidationError::Invalid)
        }
    }
}

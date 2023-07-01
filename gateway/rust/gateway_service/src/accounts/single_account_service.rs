use async_trait::async_trait;

use crate::Username;

use super::{account_service::KeyValidationError, AccountService};

pub struct SingleAccountService {
    username: Username,
    api_key: String,
}

impl SingleAccountService {
    pub fn new(username: Username, api_key: String) -> Self {
        Self { username, api_key }
    }
}

#[async_trait]
impl AccountService for SingleAccountService {
    async fn verify_user_key(
        &self,
        user: &Username,
        api_key: &str,
    ) -> Result<(), KeyValidationError> {
        if &self.username == user && self.api_key == api_key {
            Ok(())
        } else {
            Err(KeyValidationError::Invalid)
        }
    }
}

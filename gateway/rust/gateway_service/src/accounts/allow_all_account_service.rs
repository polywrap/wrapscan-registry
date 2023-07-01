use async_trait::async_trait;

use crate::Username;

use super::{account_service::KeyValidationError, AccountService};

pub struct AllowAllAccountService {}

#[async_trait]
impl AccountService for AllowAllAccountService {
    async fn verify_user_key(
        &self,
        _user: &Username,
        _api_key: &str,
    ) -> Result<(), KeyValidationError> {
        Ok(())
    }
}

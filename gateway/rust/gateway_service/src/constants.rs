pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const ENV_PACKAGES_TABLE: &str = "PACKAGES_TABLE";
pub const ENV_ACCOUNT_SERVICE_URL: &str = "ACCOUNT_SERVICE_URL";
pub const ENV_WRAP_USER_KEY: &str = "WRAP_USER_KEY";
pub const WRAP_URI_HEADER: &str = "x-wrap-uri";
pub const PACKAGES_TABLE_KEY_NAME: &str = "id";
#[cfg(feature = "local")]
pub const PACKAGES_TABLE_LOCAL: &str = "wraps-table-dev";

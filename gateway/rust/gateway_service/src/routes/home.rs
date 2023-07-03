use http::StatusCode;

use crate::constants::VERSION;

pub async fn home() -> Result<String, StatusCode> {
    let page = format!("Version: {VERSION}");

    Ok(page)
}

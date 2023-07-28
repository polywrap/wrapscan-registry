use http::{HeaderMap, StatusCode};

use crate::{debug, debugging::log_error};

pub fn extract_api_key_from_headers(headers: HeaderMap) -> Result<String, StatusCode> {
    debug!(&headers);
    // Get authentication header and validate it
    let api_key = headers
        .get("authorization")
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .to_str()
        .map_err(log_error)
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .trim_start_matches("Bearer ")
        .to_string();

    // Decode the api key
    let api_key = base64::decode(api_key)
        .map_err(log_error)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    String::from_utf8(api_key)
        .map_err(log_error)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

pub fn internal_server_error<E: std::fmt::Debug>(e: E) -> StatusCode {
    debug!(&e);
    eprintln!("INTERNAL_SERVER_ERROR serializing package: {:?}", e);
    StatusCode::INTERNAL_SERVER_ERROR
}

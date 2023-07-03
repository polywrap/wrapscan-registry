use http::{HeaderMap, StatusCode};

pub fn extract_api_key_from_headers(headers: HeaderMap) -> Result<String, StatusCode> {
    // Get authentication header and validate it
    let api_key = headers
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .trim_start_matches("Bearer ")
        .to_string();

    // Decode the api key
    let api_key = base64::decode(api_key).map_err(|_| StatusCode::UNAUTHORIZED)?;

    String::from_utf8(api_key).map_err(|_| StatusCode::UNAUTHORIZED)
}

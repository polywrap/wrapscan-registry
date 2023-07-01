use aws_sdk_dynamodb::Client;
use axum::{
    body::BoxBody,
    extract::{Path, State},
    response::Response,
    Json,
};
use http::{HeaderMap, StatusCode};

use crate::{accounts::AccountService, models::Package};
use crate::{
    accounts::RemoteAccountService, constants, dynamodb::PackageRepository, functions, Repository,
};

pub async fn publish(
    State(client): State<Client>,
    Path((user, package_and_version)): Path<(String, String)>,
    headers: HeaderMap,
    Json(UriBody { uri }): Json<UriBody>,
) -> Result<Response, StatusCode> {
    let package_repo = get_package_repository(client).await;
    let account_service = get_wrap_account_service().await;

    let api_key = extract_api_key_from_headers(headers)?;

    functions::publish(
        user,
        package_and_version,
        uri,
        api_key,
        package_repo,
        account_service,
    )
    .await?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(BoxBody::default())
        .unwrap();

    Ok(response)
}

fn extract_api_key_from_headers(headers: HeaderMap) -> Result<String, StatusCode> {
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

#[derive(serde::Deserialize)]
pub struct UriBody {
    pub uri: String,
}

async fn get_package_repository(dynamodb_client: Client) -> impl Repository<Package> {
    let table_name =
        std::env::var(constants::ENV_PACKAGES_TABLE).expect("ENV_PACKAGES_TABLE not set");

    PackageRepository::new(dynamodb_client, table_name)
}

#[cfg(not(feature = "local"))]
async fn get_wrap_account_service() -> impl AccountService {
    use crate::accounts::SingleAccountService;

    SingleAccountService::new(
        "wrap".parse().unwrap(),
        std::env::var(constants::ENV_WRAP_USER_KEY).expect("ENV_WRAP_USER_KEY not set"),
    )
}

#[cfg(feature = "local")]
async fn get_wrap_account_service() -> impl AccountService {
    use crate::AllowAllAccountService;

    AllowAllAccountService {}
}

async fn _get_remote_account_service() -> impl AccountService {
    RemoteAccountService::new(
        std::env::var(constants::ENV_ACCOUNT_SERVICE_URL).expect("ENV_ACCOUNT_SERVICE_URL not set"),
    )
}

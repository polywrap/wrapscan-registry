use axum::{
    body::BoxBody,
    extract::{Path, State},
    response::Response,
    Json,
};
use http::{HeaderMap, StatusCode};

use crate::{
    accounts::AccountService, debug_println, http_utils::extract_api_key_from_headers,
    models::Package, Repository,
};
use crate::{accounts::RemoteAccountService, constants, functions};

use super::Dependencies;

pub async fn publish<T>(
    State(deps): State<Dependencies<T>>,
    Path((user, package_and_version)): Path<(String, String)>,
    headers: HeaderMap,
    Json(UriBody { uri }): Json<UriBody>,
) -> Result<Response, StatusCode>
where
    T: Repository<Package>,
{
    let Dependencies { package_repo } = deps;

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
        .map_err(|e| {
            debug_println!("Error publishing package: {}", &e);
            eprintln!("INTERNAL_SERVER_ERROR constructing response: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(response)
}

#[derive(serde::Deserialize)]
pub struct UriBody {
    pub uri: String,
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

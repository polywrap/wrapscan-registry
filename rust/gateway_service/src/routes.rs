use aws_sdk_dynamodb::Client;

use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::get,
    Json, Router,
};
use lambda_http::{run, Error as HttpError};

use crate::{
    constants, dynamodb::PackageRepository, functions, setup_logging, AccountService, Package,
    RemoteAccountService, Repository, SingleAccountService,
};

pub async fn setup_routes() -> Result<(), HttpError> {
    setup_logging();

    let app = Router::new().route(
        "/u/:user/:packageAndVersion",
        get(resolve_package).post(publish_package),
    );

    run(app).await
}

async fn resolve_package(path: Path<(String, String)>) -> Result<Response, StatusCode> {
    let package_repo = get_package_repository().await;

    functions::resolve(path, package_repo).await
}

#[derive(serde::Deserialize)]
pub struct UriBody {
    pub uri: String,
}

async fn publish_package(
    path: Path<(String, String)>,
    headers: HeaderMap,
    body: Json<UriBody>,
) -> Result<Response, StatusCode> {
    let package_repo = get_package_repository().await;
    let account_service = get_wrap_account_service().await;

    functions::publish(path, body, headers, package_repo, account_service).await
}

async fn get_package_repository() -> impl Repository<Package> {
    let table_name =
        std::env::var(constants::ENV_PACKAGES_TABLE).expect("ENV_PACKAGES_TABLE not set");
    let client = {
        let config = aws_config::load_from_env().await;
        Client::new(&config)
    };

    PackageRepository::new(client, table_name)
}

async fn get_wrap_account_service() -> impl AccountService {
    SingleAccountService::new(
        "wrap".parse().unwrap(),
        std::env::var(constants::ENV_WRAP_USER_KEY).expect("ENV_WRAP_USER_KEY not set"),
    )
}

async fn _get_remote_account_service() -> impl AccountService {
    RemoteAccountService::new(
        std::env::var(constants::ENV_ACCOUNT_SERVICE_URL).expect("ENV_ACCOUNT_SERVICE_URL not set"),
    )
}

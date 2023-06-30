use std::fs;

use aws_config::{meta::region::RegionProviderChain, SdkConfig};
use aws_sdk_dynamodb::{Client, config::Region, meta::PKG_VERSION, types::{AttributeDefinition, ScalarAttributeType, KeySchemaElement, ProvisionedThroughput, KeyType}};

use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use lambda_http::{run, Error as HttpError};
use clap::Parser;

use crate::{
    constants, dynamodb::PackageRepository, functions, AccountService, Package,
    RemoteAccountService, Repository, SingleAccountService, setup_logging,
};

pub async fn setup_routes() -> Result<(), HttpError> {
    setup_logging();

    let app = Router::new()
        .route(
            "/dev/u/:user/:packageAndVersion/*filePath",
            get(resolve_package),
        ).route(
            "/dev/u/:user/:packageAndVersion",
            post(publish_package),
        );

    #[cfg(not(feature = "local"))]
    {
        run(app).await
    }

    #[cfg(feature = "local")]
    {      
        dotenvy::dotenv()?;

        crate::setup_local_db().await;

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();

        Ok(())
    }
}

async fn resolve_package(path: Path<(String, String, String)>) -> Result<Response, StatusCode> {
    let client = get_dynamodb_client().await;
    let package_repo = get_package_repository(client).await;

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
    let client = get_dynamodb_client().await;
    let package_repo = get_package_repository(client).await;
    let account_service = get_wrap_account_service().await;

    functions::publish(path, body, headers, package_repo, account_service).await
}

async fn get_package_repository(dynamodb_client: Client) -> impl Repository<Package> {
    let table_name =
        std::env::var(constants::ENV_PACKAGES_TABLE).expect("ENV_PACKAGES_TABLE not set");

    PackageRepository::new(dynamodb_client, table_name)
}

#[cfg(not(feature = "local"))]
async fn get_dynamodb_client() -> Client {
    let config = aws_config::load_from_env().await;
    Client::new(&config)
}

#[cfg(feature = "local")]
async fn get_dynamodb_client() -> Client {
    use crate::setup_local_db;

    let config = setup_local_db::make_config(setup_local_db::Opt::parse()).await.unwrap();
    let dynamodb_local_config = aws_sdk_dynamodb::config::Builder::from(&config)
        .endpoint_url(
            // 8000 is the default dynamodb port
            "http://localhost:8000",
        )
        .build();

    Client::from_conf(dynamodb_local_config)
}

#[cfg(not(feature = "local"))]
async fn get_wrap_account_service() -> impl AccountService {
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

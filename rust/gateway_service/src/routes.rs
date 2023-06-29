use aws_sdk_dynamodb::Client;

use axum::{extract::Path, routing::get, http::StatusCode, response::Response, Router, Json};
use lambda_http::{run, Error as HttpError};

use crate::{Repository, setup_logging, functions, dynamodb::PackageRepository, Package};

const ENV_PACKAGES_TABLE: &str = "PACKAGES_TABLE";

pub async fn setup_routes() -> Result<(), HttpError> {
    setup_logging();

    let app = Router::new()
        .route(
            "/u/:user/:packageAndVersion", 
            get(resolve_package)
                .post(publish_package)
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

async fn publish_package(path: Path<(String, String)>, body: Json<UriBody>) -> Result<Response, StatusCode> {
    let package_repo = get_package_repository().await;
   
    functions::publish(path, body, package_repo).await
}

async fn get_package_repository() -> impl Repository<Package> {
    let table_name = std::env::var(ENV_PACKAGES_TABLE).expect("ENV_PACKAGES_TABLE not set");
    let client = {
        let config = aws_config::load_from_env().await;
        Client::new(&config)
    };

    PackageRepository::new(client, table_name)
}

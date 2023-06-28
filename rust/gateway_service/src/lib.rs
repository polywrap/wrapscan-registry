mod dynamodb;
use aws_sdk_dynamodb::Client;
pub use dynamodb::*;

mod repository;
pub use repository::*;

mod models;
pub use models::*;

mod functions;
pub use functions::*;

mod publishing;

mod setup_logging;
pub use setup_logging::*;

mod semver;
pub use semver::*;

mod extract_package_and_version;
use extract_package_and_version::extract_package_and_version;

use axum::{extract::Path, routing::get, http::StatusCode, response::Response, Router, Json};
use lambda_http::{run, Error as HttpError};

const ENV_PACKAGES_TABLE: &str = "PACKAGES_TABLE";

pub async fn setup() -> Result<(), HttpError> {
    setup_logging();

    let app = Router::new()
        .route(
            "/u/:user/:packageAndVersion", 
            get(get_package)
                .post(post_package)
        );

    run(app).await
}

async fn get_package(path: Path<(String, String)>) -> Result<Response, StatusCode> {
    let package_repo = get_package_repository().await;
   
    resolve(path, package_repo).await
}

#[derive(serde::Deserialize)]
pub struct UriBody {
    pub uri: String,
}

async fn post_package(path: Path<(String, String)>, body: Json<UriBody>) -> Result<Response, StatusCode> {
    let package_repo = get_package_repository().await;
   
    publish(path, body, package_repo).await
}

async fn get_package_repository() -> impl Repository<Package> {
    let table_name = std::env::var(ENV_PACKAGES_TABLE).expect("ENV_PACKAGES_TABLE not set");
    let client = {
        let config = aws_config::load_from_env().await;
        Client::new(&config)
    };

    PackageRepository::new(client, table_name)
}

use aws_sdk_dynamodb::Client;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use lambda_http::{run, Error as HttpError};

use crate::{
    account_service::{AccountService, RemoteAccountService},
    constants,
    dynamodb::PackageRepository,
    functions, setup_logging, Package, Repository,
};

pub async fn setup_routes() -> Result<(), HttpError> {
    setup_logging();

    #[cfg(feature = "local")]
    {
        use crate::local_db;
        dotenvy::dotenv()?;

        crate::local_db::setup_local_db().await;
    }

    let dynamodb_client = get_dynamodb_client().await;

    let app = Router::new()
        .route(
            "/dev/u/:user/:packageAndVersion/*filePath",
            get(resolve_package).with_state(dynamodb_client.clone()),
        )
        .route(
            "/dev/u/:user/:packageAndVersion",
            post(publish_package).with_state(dynamodb_client.clone()),
        );

    #[cfg(not(feature = "local"))]
    {
        run(app).await
    }

    #[cfg(feature = "local")]
    {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await
            .unwrap();

        Ok(())
    }
}

async fn resolve_package(
    path: Path<(String, String, String)>,
    State(client): State<Client>,
) -> Result<Response, StatusCode> {
    let package_repo = get_package_repository(client).await;

    functions::resolve(path, package_repo).await
}

#[derive(serde::Deserialize)]
pub struct UriBody {
    pub uri: String,
}

async fn publish_package(
    State(client): State<Client>,
    path: Path<(String, String)>,
    headers: HeaderMap,
    body: Json<UriBody>,
) -> Result<Response, StatusCode> {
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

#[cfg(not(feature = "local"))]
async fn get_wrap_account_service() -> impl AccountService {
    use crate::account_service::SingleAccountService;

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

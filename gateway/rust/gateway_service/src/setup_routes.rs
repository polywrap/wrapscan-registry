use aws_sdk_dynamodb::Client;

use axum::{
    routing::{get, post},
    Router,
};
use lambda_http::{run, Error as HttpError};

use crate::{
    constants,
    dynamodb::PackageRepository,
    routes::{self, Dependencies},
    setup_logging,
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
    let table_name =
        std::env::var(constants::ENV_PACKAGES_TABLE).expect("ENV_PACKAGES_TABLE not set");
    let package_repo = PackageRepository::new(dynamodb_client, table_name);
    let deps = Dependencies {
        package_repo: package_repo.clone(),
    };

    let app = Router::new()
        .route("/", get(routes::home).with_state(deps.clone()))
        .route(
            "/r/:user/:packageAndVersion/*filePath",
            get(routes::resolve).with_state(deps.clone()),
        )
        .route(
            "/r/:user/:packageAndVersion",
            post(routes::publish).with_state(deps),
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

#[cfg(not(feature = "local"))]
async fn get_dynamodb_client() -> Client {
    let config = aws_config::load_from_env().await;
    Client::new(&config)
}

#[cfg(feature = "local")]
async fn get_dynamodb_client() -> Client {
    crate::local_db::get_dynamodb_client().await
}

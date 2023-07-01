use aws_sdk_dynamodb::Client;

use axum::{
    routing::{get, post},
    Router,
};
use lambda_http::{run, Error as HttpError};

use crate::{routes, setup_logging};

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
            get(routes::resolve).with_state(dynamodb_client.clone()),
        )
        .route(
            "/dev/u/:user/:packageAndVersion",
            post(routes::publish).with_state(dynamodb_client.clone()),
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

use aws_sdk_dynamodb::Client;

use axum::{
    routing::{get, post},
    Router,
};
use lambda_http::{run, Error as HttpError};

use crate::{
    constants,
    routes::{self, Dependencies},
    setup_logging, PackageRepository,
};

pub async fn setup_routes() -> Result<(), HttpError> {
    setup_logging();

    #[cfg(feature = "local")]
    {
        // TODO: placeholder until there's local env vars
        //dotenvy::dotenv()?;
        crate::db::local_db::setup_local_db().await;
    }

    let dynamodb_client = get_dynamodb_client().await;

    let table_name = {
        #[cfg(not(feature = "local"))]
        {
            std::env::var(constants::ENV_PACKAGES_TABLE).expect("ENV_PACKAGES_TABLE not set")
        }
        #[cfg(feature = "local")]
        {
            constants::PACKAGES_TABLE_LOCAL
        }
    };

    let package_repo = PackageRepository::new(dynamodb_client, table_name.to_owned());
    let deps = Dependencies {
        package_repo: package_repo.clone(),
    };

    let route_prefix = {
        #[cfg(not(feature = "local"))]
        {
            std::env::var(constants::ENV_STAGE)
                .expect("ENV_STAGE not set")
                .to_string()
        }
        #[cfg(feature = "local")]
        {
            "".to_string()
        }
    };

    let app = Router::new()
        .route(
            &(route_prefix.clone() + "/"),
            get(routes::home).with_state(deps.clone()),
        )
        .route(
            &(route_prefix.clone() + "/r/:user/:packageAndVersion/*filePath"),
            get(routes::resolve).with_state(deps.clone()),
        )
        .route(
            &(route_prefix + "/r/:user/:packageAndVersion"),
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
    crate::db::local_db::get_dynamodb_client().await
}

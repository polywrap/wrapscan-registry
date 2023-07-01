use aws_sdk_dynamodb::Client;
use axum::{
    body::BoxBody,
    extract::{Path, State},
    response::Response,
};
use http::StatusCode;

use crate::{constants, dynamodb::PackageRepository, functions, Package, Repository};

pub async fn resolve(
    Path((user, package_and_version, file_path)): Path<(String, String, String)>,
    State(client): State<Client>,
) -> Result<Response, StatusCode> {
    let package_repo = get_package_repository(client).await;

    let uri = functions::resolve(user, package_and_version, file_path, package_repo).await?;

    let response: Response = Response::builder()
        .status(StatusCode::OK)
        .header(constants::WRAP_URI_HEADER, uri)
        .body(BoxBody::default())
        .unwrap();

    Ok(response)
}

async fn get_package_repository(dynamodb_client: Client) -> impl Repository<Package> {
    let table_name =
        std::env::var(constants::ENV_PACKAGES_TABLE).expect("ENV_PACKAGES_TABLE not set");

    PackageRepository::new(dynamodb_client, table_name)
}

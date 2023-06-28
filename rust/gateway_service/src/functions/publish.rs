use axum::{extract::Path, http::StatusCode, response::{Response}, Json, body::BoxBody};

use crate::{Package, Repository, publishing::publish_package, UriBody, extract_package_and_version};

pub async fn publish(
    Path((user, package_and_version)): Path<(String, String)>,
    Json(body): Json<UriBody>,
    package_repo: impl Repository<Package>
) -> Result<Response, StatusCode> {
    let (package_name, version_name) = extract_package_and_version(&package_and_version);

    let uri = body.uri;

    publish_package(&user, package_name, version_name, uri, package_repo).await
        .map_err(|_| {
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(BoxBody::default())
        .unwrap();

    Ok(response)
}

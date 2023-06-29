use axum::{extract::Path, http::StatusCode, response::{Response}, body::BoxBody};

use crate::{Package, Repository, extract_package_and_version, resolving::ResolveError, resolve_package};

const WRAP_URI_HEADER: &str = "x-wrap-uri";

pub async fn resolve(
    Path((user, package_and_version)): Path<(String, String)>,
    package_repo: impl Repository<Package>
) -> Result<Response, StatusCode> {
    let (package_name, version_name) = extract_package_and_version(&package_and_version);

    let uri = resolve_package(&user, package_name, version_name, &package_repo).await
        .map_err(|e| match e {
            ResolveError::PackageNotFound => StatusCode::NOT_FOUND,
            ResolveError::VersionNotFound => StatusCode::NOT_FOUND,
            ResolveError::RepositoryError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
   
    let response: Response = Response::builder()
        .status(StatusCode::OK)
        .header(WRAP_URI_HEADER, uri)
        .body(BoxBody::default())
        .unwrap();

    Ok(response)
}

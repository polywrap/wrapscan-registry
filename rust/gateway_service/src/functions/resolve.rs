use std::fmt::Display;

use axum::{extract::Path, http::StatusCode, response::{Response}, body::BoxBody};

use crate::{Package, Repository, RepositoryError, semver, extract_package_and_version};

const WRAP_URI_HEADER: &str = "x-wrap-uri";

pub async fn resolve(
    Path((user, package_and_version)): Path<(String, String)>,
    package_repo: impl Repository<Package>
) -> Result<Response, StatusCode> {
    let (package_name, version_name) = extract_package_and_version(&package_and_version);

    let uri = resolve_package(&user, package_name, version_name, package_repo).await
        .map_err(|e| match e {
            ResolveError::PackageNotFound => StatusCode::NOT_FOUND,
            ResolveError::VersionNotFound => StatusCode::NOT_FOUND,
            ResolveError::RepositoryError => StatusCode::INTERNAL_SERVER_ERROR,
        })?;
   
    let response: Response = Response::builder()
        .status(StatusCode::OK)
        .header(WRAP_URI_HEADER, uri)
        .body(BoxBody::default())
        .unwrap();

    Ok(response)
}

#[derive(Debug, thiserror::Error)]
pub enum ResolveError {
    PackageNotFound,
    VersionNotFound,
    RepositoryError,
}
impl Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::PackageNotFound => write!(f, "Package not found"),
            ResolveError::VersionNotFound => write!(f, "Version not found"),
            ResolveError::RepositoryError => write!(f, "Repository error"),
        }
    }
}

async fn resolve_package(
    user: &str, 
    package_name: &str, 
    version_name: Option<&str>, 
    package_repo: impl Repository<Package>
) -> Result<String, ResolveError> {
    let id = format!("{}/{}", user, package_name);

    let package = package_repo
        .read(&id)
        .await
        .map_err(|error| match error {
            RepositoryError::NotFound => ResolveError::PackageNotFound,
            _ => ResolveError::RepositoryError,
        })?;

    Ok(if let Some(version) = version_name {
        let latest_version = semver::get_latest(&version, &package.versions)
            .ok_or(ResolveError::VersionNotFound)?;

        latest_version.uri.clone()
    } else {
        let mut versions = package.versions.clone();
        semver::sort_versions(&mut versions);
        let latest_version = versions.last()
            .ok_or(ResolveError::VersionNotFound)?;
    
        latest_version.uri.clone()
    })
}

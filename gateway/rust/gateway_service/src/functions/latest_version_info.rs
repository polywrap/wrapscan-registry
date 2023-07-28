use axum::http::StatusCode;

use crate::{
    debug,
    get_username_package_and_version,
    models::Package,
    resolving::{get_latest_version, ResolveError},
    Repository, http_utils::internal_server_error,
};

pub async fn latest_version_info(
    user: String,
    package_and_version: String,
    package_repo: &impl Repository<Package>,
) -> Result<String, StatusCode> {
    debug!(&user, &package_and_version);

    let (username, package_name, version_name) =
        get_username_package_and_version(user, &package_and_version)?;

    let latest_version = get_latest_version(&username, &package_name, version_name, package_repo)
        .await
        .map_err(|e| match e {
            ResolveError::PackageNotFound => StatusCode::NOT_FOUND,
            ResolveError::VersionNotFound => StatusCode::NOT_FOUND,
            ResolveError::RepositoryError(e) => internal_server_error(e),
        })?;

    let info = serde_json::to_string_pretty(&latest_version)
        .map_err(internal_server_error)?;

    Ok(info)
}

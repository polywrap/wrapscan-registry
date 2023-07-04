use axum::http::StatusCode;

use crate::{
    debug,
    debugging::log_error,
    get_username_package_and_version,
    models::Package,
    resolving::{get_latest_version, ResolveError},
    Repository,
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
        .map_err(log_error)
        .map_err(|e| match e {
            ResolveError::PackageNotFound => StatusCode::NOT_FOUND,
            ResolveError::VersionNotFound => StatusCode::NOT_FOUND,
            ResolveError::RepositoryError(e) => {
                eprintln!("INTERNAL_SERVER_ERROR resolving package: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let info = serde_json::to_string_pretty(&latest_version)
        .map_err(log_error)
        .map_err(|e| {
            eprintln!("INTERNAL_SERVER_ERROR resolving package: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(info)
}

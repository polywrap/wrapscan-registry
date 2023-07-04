use std::fmt::Display;

use crate::{
    models::{Package, PackageName, Username, Version},
    semver, Repository, RepositoryError,
};

pub async fn get_latest_version(
    user: &Username,
    package_name: &PackageName,
    version_name: Option<&str>,
    package_repo: &impl Repository<Package>,
) -> Result<Version, ResolveError> {
    let id = format!("{}/{}", user, package_name);

    let package = package_repo.read(&id).await.map_err(|error| match error {
        RepositoryError::NotFound => ResolveError::PackageNotFound,
        RepositoryError::Unknown(e) => ResolveError::RepositoryError(e),
    })?;

    Ok(if let Some(version) = version_name {
        let latest_version =
            semver::get_latest(version, &package.versions).ok_or(ResolveError::VersionNotFound)?;

        latest_version.clone()
    } else {
        let mut versions = package.versions;
        semver::sort_versions(&mut versions);
        let latest_version = versions.last().ok_or(ResolveError::VersionNotFound)?;

        latest_version.clone()
    })
}

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum ResolveError {
    PackageNotFound,
    VersionNotFound,
    RepositoryError(String),
}
impl Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolveError::PackageNotFound => write!(f, "Package not found"),
            ResolveError::VersionNotFound => write!(f, "Version not found"),
            ResolveError::RepositoryError(e) => write!(f, "Repository error: {}", e),
        }
    }
}

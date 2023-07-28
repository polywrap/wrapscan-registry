use std::fmt::Display;

use crate::{
    models::{Package, PackageName, Username},
    Repository, RepositoryError,
};

pub async fn get_package(
    user: &Username,
    package_name: &PackageName,
    package_repo: &impl Repository<Package>,
) -> Result<Package, GetPackageError> {
    let id = format!("{}/{}", user, package_name);

    let package = package_repo.read(&id).await.map_err(|error| match error {
        RepositoryError::NotFound => GetPackageError::PackageNotFound,
        RepositoryError::Unknown(e) => GetPackageError::RepositoryError(e),
    })?;

    Ok(package)
}

#[derive(Debug, thiserror::Error, PartialEq, Clone)]
pub enum GetPackageError {
    PackageNotFound,
    RepositoryError(String),
}
impl Display for GetPackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GetPackageError::PackageNotFound => write!(f, "Package not found"),
            GetPackageError::RepositoryError(e) => write!(f, "Repository error: {}", e),
        }
    }
}

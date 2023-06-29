use std::fmt::Display;

use crate::{semver, Package, Repository, RepositoryError, package_name::PackageName, username::Username};

pub async fn resolve_package(
    user: &Username,
    package_name: &PackageName,
    version_name: Option<&str>,
    package_repo: &impl Repository<Package>,
) -> Result<String, ResolveError> {
    let id = format!("{}/{}", user, package_name);

    let package = package_repo.read(&id).await
        .map_err(|error| match error {
            RepositoryError::NotFound => ResolveError::PackageNotFound,
            RepositoryError::Unknown(e) => ResolveError::RepositoryError(e.to_string()),
        })?;

    Ok(if let Some(version) = version_name {
        let latest_version =
            semver::get_latest(&version, &package.versions).ok_or(ResolveError::VersionNotFound)?;

        latest_version.uri.clone()
    } else {
        let mut versions = package.versions.clone();
        semver::sort_versions(&mut versions);
        let latest_version = versions.last().ok_or(ResolveError::VersionNotFound)?;

        latest_version.uri.clone()
    })
}

#[derive(Debug, thiserror::Error, PartialEq)]
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

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::{mock, predicate::eq};
    use resolve_package::ResolveError;

    use crate::{resolving::resolve_package, Package, Repository, RepositoryError, Version, username::Username, package_name::PackageName};

    mock! {
      PackageRepository {}
        #[async_trait]
        impl Repository<Package> for PackageRepository {
            async fn read(&self, key: &str) -> Result<Package, RepositoryError>;
            async fn update(&self, entity: &Package) -> Result<(), RepositoryError>;
        }
    }

    #[tokio::test]
    async fn can_resolve_package() {
        let mut mock_repo = MockPackageRepository::new();

        let user: Username = "user1".parse().unwrap();
        let package_name: PackageName = "package1".parse().unwrap();
        let id = format!("{}/{}", user, package_name);

        let expected_package = Package {
            id: id.clone(),
            user: user.clone(),
            name: package_name.clone(),
            versions: vec![
                Version {
                    name: "1.0.0".to_string(),
                    uri: "uri1".to_string(),
                },
                Version {
                    name: "2.0.0".to_string(),
                    uri: "uri2".to_string(),
                },
            ],
        };

        mock_repo
            .expect_read()
            .with(eq(id.clone()))
            .times(1)
            .returning(move |_| Ok(expected_package.clone()));

        let result = resolve_package(&user, &package_name, None, &mock_repo).await;

        assert_eq!(result, Ok("uri2".to_string()));
    }

    #[tokio::test]
    async fn resolves_package_with_specified_version() {
        let mut mock_repo = MockPackageRepository::new();

        let user: Username = "user1".parse().unwrap();
        let package_name: PackageName = "package1".parse().unwrap();
        let id = format!("{}/{}", user, package_name);

        let expected_package = Package {
            id: id.clone(),
            user: user.clone(),
            name: package_name.clone(),
            versions: vec![
                Version {
                    name: "1.0.0".to_string(),
                    uri: "uri1".to_string(),
                },
                Version {
                    name: "2.0.0".to_string(),
                    uri: "uri2".to_string(),
                },
            ],
        };

        mock_repo
            .expect_read()
            .with(eq(id.clone()))
            .times(1)
            .returning(move |_| Ok(expected_package.clone()));

        let result = resolve_package(&user, &package_name, Some("2.0.0"), &mock_repo).await;

        assert_eq!(result, Ok("uri2".to_string()));
    }

    #[tokio::test]
    async fn returns_version_not_found_error_when_resolving_package_with_non_existent_version() {
        let mut mock_repo = MockPackageRepository::new();

        let user: Username = "user1".parse().unwrap();
        let package_name: PackageName = "package1".parse().unwrap();
        let id = format!("{}/{}", user, package_name);

        let expected_package = Package {
            id: id.clone(),
            user: user.clone(),
            name: package_name.clone(),
            versions: vec![
                Version {
                    name: "1.0.0".to_string(),
                    uri: "uri1".to_string(),
                },
                Version {
                    name: "2.0.0".to_string(),
                    uri: "uri2".to_string(),
                },
            ],
        };

        mock_repo
            .expect_read()
            .with(eq(id.clone()))
            .times(1)
            .returning(move |_| Ok(expected_package.clone()));

        let result = resolve_package(&user, &package_name, Some("3.0.0"), &mock_repo).await;

        assert_eq!(result, Err(ResolveError::VersionNotFound));
    }

    #[tokio::test]
    async fn returns_package_not_found_when_resolving_non_existent_package() {
        let mut mock_repo = MockPackageRepository::new();

        let user: Username = "user1".parse().unwrap();
        let package_name: PackageName = "package1".parse().unwrap();
        let id = format!("{}/{}", user, package_name);

        mock_repo
            .expect_read()
            .with(eq(id.clone()))
            .times(1)
            .returning(move |_| Err(RepositoryError::NotFound));

        let result = resolve_package(&user, &package_name, None, &mock_repo).await;

        assert_eq!(result, Err(ResolveError::PackageNotFound));
    }

    #[tokio::test]
    async fn returns_repository_error_when_resolving_package_with_repository_error() {
        let mut mock_repo = MockPackageRepository::new();

        let user: Username = "user1".parse().unwrap();
        let package_name: PackageName = "package1".parse().unwrap();
        let id = format!("{}/{}", user, package_name);

        mock_repo
            .expect_read()
            .with(eq(id.clone()))
            .times(1)
            .returning(move |_| Err(RepositoryError::Unknown("Some error".to_string())));

        let result = resolve_package(&user, &package_name, None, &mock_repo).await;

        assert_eq!(
            result,
            Err(ResolveError::RepositoryError("Some error".to_string()))
        );
    }
}

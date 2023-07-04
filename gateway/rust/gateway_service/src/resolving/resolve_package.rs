use crate::{
    models::{Package, PackageName, Username, WrapUri},
    Repository,
};

use super::ResolveError;

pub async fn resolve_package(
    user: &Username,
    package_name: &PackageName,
    version_name: Option<&str>,
    package_repo: &impl Repository<Package>,
) -> Result<WrapUri, ResolveError> {
    let latest_version =
        super::get_latest_version(user, package_name, version_name, package_repo).await?;

    Ok(latest_version.uri)
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::{mock, predicate::eq};
    use resolve_package::ResolveError;

    use crate::{
        models::{Package, PackageName, Username, Version},
        resolving::resolve_package,
        Repository, RepositoryError,
    };

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
                    uri: "test/uri1".parse().unwrap(),
                    created_on: 0,
                },
                Version {
                    name: "2.0.0".to_string(),
                    uri: "test/uri2".parse().unwrap(),
                    created_on: 0,
                },
            ],
            created_on: 0,
        };

        mock_repo
            .expect_read()
            .with(eq(id.clone()))
            .times(1)
            .returning(move |_| Ok(expected_package.clone()));

        let result = resolve_package(&user, &package_name, None, &mock_repo).await;

        assert_eq!(result, Ok("test/uri2".parse().unwrap()));
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
                    uri: "test/uri1".parse().unwrap(),
                    created_on: 0,
                },
                Version {
                    name: "2.0.0".to_string(),
                    uri: "test/uri2".parse().unwrap(),
                    created_on: 0,
                },
            ],
            created_on: 0,
        };

        mock_repo
            .expect_read()
            .with(eq(id.clone()))
            .times(1)
            .returning(move |_| Ok(expected_package.clone()));

        let result = resolve_package(&user, &package_name, Some("2.0.0"), &mock_repo).await;

        assert_eq!(result, Ok("test/uri2".parse().unwrap()));
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
                    uri: "test/uri1".parse().unwrap(),
                    created_on: 0,
                },
                Version {
                    name: "2.0.0".to_string(),
                    uri: "test/uri2".parse().unwrap(),
                    created_on: 0,
                },
            ],
            created_on: 0,
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

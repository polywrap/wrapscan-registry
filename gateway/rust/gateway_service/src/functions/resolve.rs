use axum::http::StatusCode;

use crate::{
    debug, debug_println, get_username_package_and_version, resolve_package,
    resolving::ResolveError, Package, Repository, WrapUri,
};

pub async fn resolve(
    user: String,
    package_and_version: String,
    _file_path: String,
    package_repo: impl Repository<Package>,
) -> Result<WrapUri, StatusCode> {
    debug!(&user, &package_and_version, &_file_path);

    let (username, package_name, version_name) =
        get_username_package_and_version(user, &package_and_version)?;

    let uri = resolve_package(&username, &package_name, version_name, &package_repo)
        .await
        .map_err(|e| {
            debug_println!("Error resolving package: {}", &e);
            match e {
                ResolveError::PackageNotFound => StatusCode::NOT_FOUND,
                ResolveError::VersionNotFound => StatusCode::NOT_FOUND,
                ResolveError::RepositoryError(e) => {
                    eprintln!("INTERNAL_SERVER_ERROR resolving package: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        })?;

    Ok(uri)
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::http::StatusCode;
    use mockall::{mock, predicate::eq};

    use crate::{functions::resolve, Package, Repository, RepositoryError, Version};

    mock! {
      PackageRepository {}
        #[async_trait]
        impl Repository<Package> for PackageRepository {
            async fn read(&self, key: &str) -> Result<Package, RepositoryError>;
            async fn update(&self, entity: &Package) -> Result<(), RepositoryError>;
        }
    }

    #[tokio::test]
    async fn can_resolve_latest_version() {
        let mut package_repo = MockPackageRepository::new();

        let package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![
                Version {
                    name: "1.0.0".into(),
                    uri: "test/uri0".parse().unwrap(),
                },
                Version {
                    name: "1.0.1".into(),
                    uri: "test/uri1".parse().unwrap(),
                },
                Version {
                    name: "1.0.2".into(),
                    uri: "test/uri2".parse().unwrap(),
                },
            ],
        };

        package_repo
            .expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| Ok(package));

        let result = resolve(
            "user1".into(),
            "package1".into(),
            "some/path".into(),
            package_repo,
        )
        .await
        .unwrap();

        assert_eq!(result, "test/uri2".parse().unwrap());
    }

    #[tokio::test]
    async fn can_resolve_specific_version() {
        let mut package_repo = MockPackageRepository::new();

        let package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![
                Version {
                    name: "1.0.0".into(),
                    uri: "test/uri0".parse().unwrap(),
                },
                Version {
                    name: "1.0.1".into(),
                    uri: "test/uri1".parse().unwrap(),
                },
                Version {
                    name: "1.0.2".into(),
                    uri: "test/uri2".parse().unwrap(),
                },
            ],
        };

        package_repo
            .expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| Ok(package));

        let result = resolve(
            "user1".into(),
            "package1@1.0.1".into(),
            "some/path".into(),
            package_repo,
        )
        .await
        .unwrap();

        assert_eq!(result, "test/uri1".parse().unwrap());
    }

    #[tokio::test]
    async fn resolve_package_not_found() {
        let mut package_repo = MockPackageRepository::new();

        package_repo
            .expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| Err(RepositoryError::NotFound));

        let result = resolve(
            "user1".into(),
            "package1".into(),
            "some/path".into(),
            package_repo,
        )
        .await;

        assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
    }

    #[tokio::test]
    async fn resolve_version_not_found() {
        let mut package_repo = MockPackageRepository::new();

        let package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![Version {
                name: "1.0.0".into(),
                uri: "test/uri1".parse().unwrap(),
            }],
        };

        package_repo
            .expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| Ok(package));

        let result = resolve(
            "user1".into(),
            "package1@1.0.1".into(),
            "some/path".into(),
            package_repo,
        )
        .await;

        assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
    }

    #[tokio::test]
    async fn invalid_package_name_returns_bad_request1() {
        let mut package_repo = MockPackageRepository::new();

        package_repo.expect_read().times(0);

        let result = resolve(
            "user1".into(),
            "pack!age1@1.0.0".into(),
            "some/path".into(),
            package_repo,
        )
        .await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }

    #[tokio::test]
    async fn invalid_package_name_returns_bad_request2() {
        let mut package_repo = MockPackageRepository::new();

        package_repo.expect_read().times(0);

        let result = resolve(
            "user1".into(),
            "pack age1@1.0.0".into(),
            "some/path".into(),
            package_repo,
        )
        .await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }
}

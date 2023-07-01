use crate::{Package, Repository, Version, WrapUri};

use super::error::PublishError;

pub async fn publish_latest_version(
    package: &mut Package,
    uri: WrapUri,
    package_repo: impl Repository<Package>,
) -> Result<(), PublishError> {
    if package.versions.len() > 1 {
        return Err(PublishError::LatestVersionNotAllowed);
    }

    if package.versions.len() == 1 {
        let existing_version = &mut package.versions[0];

        if existing_version.name != "latest" {
            return Err(PublishError::LatestVersionNotAllowed);
        }

        existing_version.uri = uri;
    } else {
        package.versions.push(Version {
            name: "latest".to_string(),
            uri: uri.clone(),
        });
    }

    package_repo
        .update(package)
        .await
        .map_err(|e| PublishError::RepositoryError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::{mock, predicate::eq};

    use crate::{
        publishing::{publish_latest_version, PublishError},
        Package, Repository, RepositoryError, Version,
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
    async fn can_publish_latest_version_when_no_versions_published() {
        let mut package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![],
        };

        let update_package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![Version {
                name: "latest".into(),
                uri: "test/uri_latest".parse().unwrap(),
            }],
        };

        let mut mock_package_repo = MockPackageRepository::new();
        mock_package_repo
            .expect_update()
            .with(eq(update_package))
            .times(1)
            .returning(|_| Ok(()));

        let result = publish_latest_version(
            &mut package,
            "test/uri_latest".parse().unwrap(),
            mock_package_repo,
        )
        .await;

        assert!(
            result.is_ok(),
            "Publishing the latest version failed: {:?}",
            result
        );
        assert_eq!(package.versions.len(), 1, "Unexpected number of versions");
        assert_eq!(
            package.versions[0].name, "latest",
            "Version name is not 'latest'"
        );
        assert_eq!(
            package.versions[0].uri,
            "test/uri_latest".parse().unwrap(),
            "Unexpected URI for the latest version"
        );
    }

    #[tokio::test]
    async fn can_publish_latest_version_when_latest_already_published() {
        let mut package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![Version {
                name: "latest".into(),
                uri: "test/uri1".parse().unwrap(),
            }],
        };

        let update_package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![Version {
                name: "latest".into(),
                uri: "test/uri_latest".parse().unwrap(),
            }],
        };

        let mut mock_package_repo = MockPackageRepository::new();
        mock_package_repo
            .expect_update()
            .with(eq(update_package.clone()))
            .times(1)
            .returning(|_| Ok(()));

        let result = publish_latest_version(
            &mut package,
            "test/uri_latest".parse().unwrap(),
            mock_package_repo,
        )
        .await;

        assert!(
            result.is_ok(),
            "Publishing the latest version failed: {:?}",
            result
        );
        assert_eq!(package.versions.len(), 1, "Unexpected number of versions");
        assert_eq!(
            package.versions[0].name, "latest",
            "Version name is not 'latest'"
        );
        assert_eq!(
            package.versions[0].uri,
            "test/uri_latest".parse().unwrap(),
            "Unexpected URI for the latest version"
        );
    }

    #[tokio::test]
    async fn can_publish_latest_version_when_one_already_published() {
        let mut package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![Version {
                name: "1.0.0".into(),
                uri: "test/uri1".parse().unwrap(),
            }],
        };

        let mut mock_package_repo = MockPackageRepository::new();
        mock_package_repo.expect_update().times(0);

        let result = publish_latest_version(
            &mut package,
            "test/uri_latest".parse().unwrap(),
            mock_package_repo,
        )
        .await;

        assert_eq!(result, Err(PublishError::LatestVersionNotAllowed));
    }

    #[tokio::test]
    async fn can_publish_latest_version_when_multiple_already_published() {
        let mut package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![
                Version {
                    name: "1.0.0".into(),
                    uri: "test/uri1".parse().unwrap(),
                },
                Version {
                    name: "1.0.1".into(),
                    uri: "test/uri2".parse().unwrap(),
                },
            ],
        };

        let mut mock_package_repo = MockPackageRepository::new();
        mock_package_repo.expect_update().times(0);

        let result = publish_latest_version(
            &mut package,
            "test/uri_latest".parse().unwrap(),
            mock_package_repo,
        )
        .await;

        assert_eq!(result, Err(PublishError::LatestVersionNotAllowed));
    }

    #[tokio::test]
    async fn publish_latest_version_fails_when_unknown_repository_error() {
        let mut package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![],
        };

        let update_package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![Version {
                name: "latest".into(),
                uri: "test/uri_latest".parse().unwrap(),
            }],
        };

        let mut mock_package_repo = MockPackageRepository::new();
        mock_package_repo
            .expect_update()
            .with(eq(update_package))
            .times(1)
            .returning(|_| Err(RepositoryError::Unknown("some error".to_string())));

        let result = publish_latest_version(
            &mut package,
            "test/uri_latest".parse().unwrap(),
            mock_package_repo,
        )
        .await;

        assert_eq!(
            result,
            Err(PublishError::RepositoryError(
                RepositoryError::Unknown("some error".to_string()).to_string()
            ))
        );
    }
}

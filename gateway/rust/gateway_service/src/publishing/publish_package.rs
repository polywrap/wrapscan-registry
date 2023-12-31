use crate::models::{Package, PackageName, PartialVersion, Username, Version, WrapUri};
use crate::{semver, Repository, RepositoryError};

use super::error::PublishError;

use super::publish_latest_version;

pub async fn publish_package(
    user: &Username,
    package_name: &PackageName,
    version_name: Option<&str>,
    uri: WrapUri,
    package_repo: impl Repository<Package>,
) -> Result<(), PublishError> {
    if let Some(version) = version_name {
        if !semver::verify(&PartialVersion {
            name: version.to_string(),
        }) {
            return Err(PublishError::InvalidVersionFormat);
        }
    }

    let new_version = version_name.unwrap_or("latest");
    let id = format!("{}/{}", user, package_name);

    let package = package_repo.read(&id).await;

    let package = match package {
        Ok(package) => Some(package),
        Err(RepositoryError::NotFound) => None,
        Err(e) => return Err(PublishError::RepositoryError(e.to_string())),
    };

    let mut package = if let Some(mut package) = package {
        if new_version == "latest" {
            return publish_latest_version(&mut package, uri, package_repo).await;
        }

        let existing_version = package
            .versions
            .iter()
            .find(|version| version.name == new_version);
        if let Some(existing_version) = existing_version {
            match existing_version.uri == uri {
                true => return Err(PublishError::DuplicateVersionNameAndUri),
                false => return Err(PublishError::DuplicateVersionName),
            }
        }

        package
    } else {
        Package::new(package_name.clone(), user.clone())
    };

    package
        .versions
        .push(Version::new(new_version.to_string(), uri));

    semver::sort_versions(&mut package.versions);

    package_repo
        .update(&package)
        .await
        .map_err(|e| PublishError::RepositoryError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::{mock, predicate::eq};

    use crate::{
        publishing::{publish_package, PublishError},
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
    async fn can_publish_version() {
        let package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![Version {
                name: "1.0.0".into(),
                uri: "test/uri1".parse().unwrap(),
                created_on: 0,
            }],
            created_on: 0,
        };

        let new_version = Version {
            name: "2.0.0".into(),
            uri: "test/uri2".parse().unwrap(),
            created_on: 0,
        };

        let mut package_repo = MockPackageRepository::new();

        let read_package = package.clone();
        package_repo
            .expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| Ok(read_package));

        let package = package.clone();
        package_repo
            .expect_update()
            .withf(move |p| {
                p.id == package.id && p.versions.len() == 2 && p.versions[1] == new_version
            })
            .return_once(move |_| Ok(()));

        let result = publish_package(
            &package.user,
            &package.name,
            Some("2.0.0"),
            "test/uri2".parse().unwrap(),
            package_repo,
        )
        .await;

        assert_eq!(result, Ok(()));
    }

    #[tokio::test]
    async fn forbids_publishing_duplicate_version_name() {
        let package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![Version {
                name: "1.0.0".into(),
                uri: "test/uri1".parse().unwrap(),
                created_on: 0,
            }],
            created_on: 0,
        };

        let mut package_repo = MockPackageRepository::new();

        {
            let package = package.clone();
            package_repo
                .expect_read()
                .with(eq("user1/package1".to_string()))
                .return_once(move |_| Ok(package));
        }

        package_repo.expect_update().never();

        let result = publish_package(
            &package.user,
            &package.name,
            Some("1.0.0"),
            "test/uri2".parse().unwrap(),
            package_repo,
        )
        .await;

        assert_eq!(result, Err(PublishError::DuplicateVersionName));
    }

    #[tokio::test]
    async fn forbids_publishing_duplicate_version_name_and_uri() {
        let package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![Version {
                name: "1.0.0".into(),
                uri: "test/uri1".parse().unwrap(),
                created_on: 0,
            }],
            created_on: 0,
        };

        let mut package_repo = MockPackageRepository::new();

        {
            let package = package.clone();
            package_repo
                .expect_read()
                .with(eq("user1/package1".to_string()))
                .return_once(move |_| Ok(package));
        }

        package_repo.expect_update().never();

        let result = publish_package(
            &package.user,
            &package.name,
            Some("1.0.0"),
            "test/uri1".parse().unwrap(),
            package_repo,
        )
        .await;

        assert_eq!(result, Err(PublishError::DuplicateVersionNameAndUri));
    }

    #[tokio::test]
    async fn forbids_publishing_invalid_version() {
        let package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![Version {
                name: "1.0.0".into(),
                uri: "test/uri1".parse().unwrap(),
                created_on: 0,
            }],
            created_on: 0,
        };

        let mut package_repo = MockPackageRepository::new();

        {
            let package = package.clone();
            package_repo
                .expect_read()
                .with(eq("user1/package1".to_string()))
                .return_once(move |_| Ok(package));
        }
        package_repo.expect_update().never();

        let result = publish_package(
            &package.user,
            &package.name,
            Some("1.0.0a"),
            "test/uri2".parse().unwrap(),
            package_repo,
        )
        .await;

        assert_eq!(result, Err(PublishError::InvalidVersionFormat));
    }
}

use axum::http::StatusCode;

use crate::{
    debug,
    http_utils::internal_server_error,
    models::{Package, PackageName, Username},
    resolving::{get_package, GetPackageError},
    Repository,
};

pub async fn package_info(
    user: String,
    package: String,
    package_repo: &impl Repository<Package>,
) -> Result<String, StatusCode> {
    debug!(&user, &package);

    let username: Username = user.parse().map_err(internal_server_error)?;

    let package_name: PackageName = package.parse().map_err(internal_server_error)?;

    let package = get_package(&username, &package_name, package_repo)
        .await
        .map_err(|e| match e {
            GetPackageError::PackageNotFound => StatusCode::NOT_FOUND,
            GetPackageError::RepositoryError(e) => internal_server_error(e),
        })?;

    Ok(serde_json::to_string_pretty(&package).map_err(internal_server_error)?)
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::{mock, predicate::eq};

    use crate::{functions::package_info, Package, Repository, RepositoryError, Version};

    mock! {
      PackageRepository {}
        #[async_trait]
        impl Repository<Package> for PackageRepository {
            async fn read(&self, key: &str) -> Result<Package, RepositoryError>;
            async fn update(&self, entity: &Package) -> Result<(), RepositoryError>;
        }
    }

    #[tokio::test]
    async fn can_get_package_info() {
        let mut package_repo = MockPackageRepository::new();

        let package = Package {
            id: "user1/package1".into(),
            name: "package1".parse().unwrap(),
            user: "user1".parse().unwrap(),
            versions: vec![
                Version {
                    name: "1.0.0".into(),
                    uri: "test/uri0".parse().unwrap(),
                    created_on: 0,
                },
                Version {
                    name: "1.0.1".into(),
                    uri: "test/uri1".parse().unwrap(),
                    created_on: 0,
                },
                Version {
                    name: "1.0.2".into(),
                    uri: "test/uri2".parse().unwrap(),
                    created_on: 0,
                },
            ],
            created_on: 0,
        };

        {
            let package = package.clone();
            package_repo
                .expect_read()
                .with(eq("user1/package1".to_string()))
                .return_once(move |_| Ok(package));
        }

        let result = package_info("user1".into(), "package1".into(), &package_repo)
            .await
            .unwrap();

        assert_eq!(result, serde_json::to_string_pretty(&package).unwrap());
    }
}

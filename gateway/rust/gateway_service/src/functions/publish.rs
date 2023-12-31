use axum::http::StatusCode;

use crate::{
    accounts::KeyValidationError,
    debug, debug_println,
    debugging::log_error,
    get_username_package_and_version,
    models::Package,
    publishing::{publish_package, PublishError},
    AccountService, Repository, http_utils::internal_server_error,
};

pub async fn publish(
    user: String,
    package_and_version: String,
    uri: String,
    api_key: String,
    package_repo: impl Repository<Package>,
    account_service: impl AccountService,
) -> Result<(), StatusCode> {
    debug!(&user, &package_and_version, &uri, &api_key);

    let (username, package_name, version_name) =
        get_username_package_and_version(user, &package_and_version)?;

    let uri = uri
        .parse()
        .map_err(log_error)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    debug_println!("Verifying API key: {:?}", &api_key);

    account_service
        .verify_user_key(&username, &api_key)
        .await
        .map_err(log_error)
        .map_err(|e| match e {
            KeyValidationError::Invalid => StatusCode::UNAUTHORIZED,
            KeyValidationError::Unknown(e) => internal_server_error(e)
        })?;

    debug_println!("Publishing package: {:?}", &package_name);

    publish_package(&username, &package_name, version_name, uri, package_repo)
        .await
        .map_err(log_error)
        .map_err(|e| match e {
            PublishError::InvalidVersionFormat => StatusCode::BAD_REQUEST,
            PublishError::DuplicateVersionName => StatusCode::BAD_REQUEST,
            // If the version name and URI are the same, then we can just return OK since nothing needs to be change.
            PublishError::DuplicateVersionNameAndUri => StatusCode::OK,
            PublishError::LatestVersionNotAllowed => StatusCode::BAD_REQUEST,
            PublishError::RepositoryError(e) => internal_server_error(e),
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use mockall::{mock, predicate::eq};

    use crate::{
        functions::publish,
        models::{Package, Username},
        AccountService, KeyValidationError, Repository, RepositoryError, Version,
    };

    mock! {
      PackageRepository {}
        #[async_trait]
        impl Repository<Package> for PackageRepository {
            async fn read(&self, key: &str) -> Result<Package, RepositoryError>;
            async fn update(&self, entity: &Package) -> Result<(), RepositoryError>;
        }
    }

    mock! {
        AccountService {}
        #[async_trait]
        impl AccountService for AccountService {
            async fn verify_user_key(&self, username: &Username, api_key: &str) -> Result<(), KeyValidationError>;
        }
    }

    #[tokio::test]
    async fn publish_version() {
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
        let mut account_service = MockAccountService::new();

        account_service
            .expect_verify_user_key()
            .with(eq(package.user.clone()), eq("key1"))
            .return_once(|_, _| Ok(()));

        {
            let package = package.clone();
            package_repo
                .expect_read()
                .with(eq("user1/package1".to_string()))
                .return_once(move |_| Ok(package));
        }
        {
            let package = package.clone();
            package_repo
                .expect_update()
                .withf(move |p| {
                    p.id == package.id && p.versions.len() == 2 && p.versions[1] == new_version
                })
                .return_once(move |_| Ok(()));
        }

        publish(
            "user1".into(),
            "package1@2.0.0".into(),
            "test/uri2".parse().unwrap(),
            "key1".into(),
            package_repo,
            account_service,
        )
        .await
        .unwrap();
    }
}

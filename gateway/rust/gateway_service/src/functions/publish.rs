use axum::http::StatusCode;

use crate::{
    accounts::KeyValidationError,
    debug, debug_println, get_username_package_and_version,
    models::Package,
    publishing::{publish_package, PublishError},
    AccountService, Repository,
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

    let uri = uri.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    account_service
        .verify_user_key(&username, &api_key)
        .await
        .map_err(|e| {
            debug_println!("Error publishing package: {}", &e);

            match e {
                KeyValidationError::Invalid => StatusCode::UNAUTHORIZED,
                KeyValidationError::Unknown(e) => {
                    eprintln!("INTERNAL_SERVER_ERROR verifying user key: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
        })?;

    publish_package(&username, &package_name, version_name, uri, package_repo)
        .await
        .map_err(|e| {
            debug_println!("Error publishing package: {}", &e);

            match e {
                PublishError::InvalidVersionFormat => StatusCode::BAD_REQUEST,
                PublishError::DuplicateVersion => StatusCode::BAD_REQUEST,
                PublishError::LatestVersionNotAllowed => StatusCode::BAD_REQUEST,
                PublishError::RepositoryError(e) => {
                    eprintln!("INTERNAL_SERVER_ERROR publishing package: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            }
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

use axum::{
    body::BoxBody,
    extract::Path,
    http::{HeaderMap, StatusCode},
    response::Response,
    Json,
};

use crate::{
    extract_package_and_version,
    package_name::PackageName,
    publishing::{publish_package, PublishError},
    routes::UriBody,
    username::Username,
    AccountService, KeyValidationError, Package, Repository,
};

pub async fn publish(
    Path((user, package_and_version)): Path<(String, String)>,
    Json(UriBody { uri }): Json<UriBody>,
    headers: HeaderMap,
    package_repo: impl Repository<Package>,
    account_service: impl AccountService,
) -> Result<Response, StatusCode> {
    let (username, package_name, version_name) =
        build_username_package_and_version(user, &package_and_version)?;

    let api_key = get_api_key(headers)?;

    account_service
        .verify_user_key(&username, &api_key)
        .await
        .map_err(|e| match e {
            KeyValidationError::Invalid => StatusCode::UNAUTHORIZED,
            KeyValidationError::Unknown(e) => {
                eprintln!("INTERNAL_SERVER_ERROR verifying user key: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    publish_package(&username, &package_name, version_name, uri, package_repo)
        .await
        .map_err(|e| match e {
            PublishError::InvalidVersionFormat => StatusCode::BAD_REQUEST,
            PublishError::DuplicateVersion => StatusCode::BAD_REQUEST,
            PublishError::LatestVersionNotAllowed => StatusCode::BAD_REQUEST,
            PublishError::RepositoryError(e) => {
                eprintln!("INTERNAL_SERVER_ERROR publishing package: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(BoxBody::default())
        .unwrap();

    Ok(response)
}

fn build_username_package_and_version(
    user: String,
    package_and_version: &str,
) -> Result<(Username, PackageName, Option<&str>), StatusCode> {
    let username = user.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    let (package_name, version_name) = extract_package_and_version(&package_and_version);

    let package_name = package_name.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok((username, package_name, version_name))
}

fn get_api_key(headers: HeaderMap) -> Result<String, StatusCode> {
    // Get authentication header and validate it
    let api_key = headers
        .get("Authorization")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::UNAUTHORIZED)?
        .trim_start_matches("Bearer ")
        .to_string();

    // Decode the api key
    let api_key = base64::decode(api_key).map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(String::from_utf8(api_key).map_err(|_| StatusCode::UNAUTHORIZED)?)
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::{
        extract::Path,
        http::{HeaderMap, StatusCode},
        Json,
    };
    use mockall::{mock, predicate::eq};

    use crate::{
        account_service, functions::publish, package_name::PackageName, routes::UriBody,
        username::Username, AccountService, KeyValidationError, Package, Repository,
        RepositoryError, Version,
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
                uri: "uri1".into(),
            }],
        };

        let new_version = Version {
            name: "2.0.0".into(),
            uri: "uri2".into(),
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
                .return_once(move |_| Ok(package.clone()));
        }
        {
            let package = package.clone();
            package_repo
                .expect_update()
                .withf(move |p| {
                    &p.id == &package.id && p.versions.len() == 2 && p.versions[1] == new_version
                })
                .return_once(move |_| Ok(()));
        }

        let body: Json<UriBody> = Json(UriBody { uri: "uri2".into() });
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            format!("Bearer {}", base64::encode("key1"))
                .parse()
                .unwrap(),
        );

        let result = publish(
            Path(("user1".into(), "package1@2.0.0".into())),
            body,
            headers,
            package_repo,
            account_service,
        )
        .await
        .unwrap();

        assert!(matches!(result.status(), StatusCode::OK));
    }
}

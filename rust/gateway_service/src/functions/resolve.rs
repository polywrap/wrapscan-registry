use axum::{body::BoxBody, extract::Path, http::StatusCode, response::Response};

use crate::{
    constants, extract_package_and_version, resolve_package, resolving::ResolveError, Package,
    Repository, username::Username, package_name::PackageName,
};

pub async fn resolve(
    Path((user, package_and_version)): Path<(String, String)>,
    package_repo: impl Repository<Package>,
) -> Result<Response, StatusCode> {
    let (username, package_name, version_name) = build_username_package_and_version(user, &package_and_version)?;

    let uri = resolve_package(&username, &package_name, version_name, &package_repo)
        .await
        .map_err(|e| match e {
            ResolveError::PackageNotFound => StatusCode::NOT_FOUND,
            ResolveError::VersionNotFound => StatusCode::NOT_FOUND,
            ResolveError::RepositoryError(e) => {
                eprintln!("INTERNAL_SERVER_ERROR resolving package: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    let response: Response = Response::builder()
        .status(StatusCode::OK)
        .header(constants::WRAP_URI_HEADER, uri)
        .body(BoxBody::default())
        .unwrap();

    Ok(response)
}

fn build_username_package_and_version(user: String, package_and_version: &str) -> Result<(Username, PackageName, Option<&str>), StatusCode> {
    let username = Username::try_from(user)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let (package_name, version_name) = extract_package_and_version(&package_and_version);

    let package_name = PackageName::try_from(package_name.to_string())
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok((username, package_name, version_name))
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::{extract::Path, http::StatusCode, Json};
    use mockall::{mock, predicate::eq};

    use crate::{functions::resolve, Package, Repository, RepositoryError, UriResponse, Version, package_name::PackageName, username::Username};

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
            name: PackageName::try_from("package1".to_string()).unwrap(),
            user: Username::try_from("user1".to_string()).unwrap(),
            versions: vec![
                Version {
                    name: "1.0.0".into(),
                    uri: "uri0".into(),
                },
                Version {
                    name: "1.0.1".into(),
                    uri: "uri1".into(),
                },
                Version {
                    name: "1.0.2".into(),
                    uri: "uri2".into(),
                }
            ],
        };

        package_repo
            .expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| Ok(package.clone()));

        let result = resolve(
            Path(("user1".into(), "package1".into())),
            package_repo,
        )
        .await;

        let expected_response = UriResponse {
            uri: "uri2".to_string(),
        };

        let _: Result<Json<UriResponse>, StatusCode> = Ok(Json(expected_response));

        assert!(matches!(result, _));
    }

    #[tokio::test]
    async fn can_resolve_specific_version() {
        let mut package_repo = MockPackageRepository::new();

        let package = Package {
            id: "user1/package1".into(),
            name: PackageName::try_from("package1".to_string()).unwrap(),
            user: Username::try_from("user1".to_string()).unwrap(),
            versions: vec![
                Version {
                    name: "1.0.0".into(),
                    uri: "uri0".into(),
                },
                Version {
                    name: "1.0.1".into(),
                    uri: "uri1".into(),
                },
                Version {
                    name: "1.0.2".into(),
                    uri: "uri2".into(),
                }
            ],
        };

        package_repo
            .expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| Ok(package.clone()));

        let result = resolve(
            Path(("user1".into(), "package1@1.0.1".into())),
            package_repo,
        )
        .await;

        let expected_response = UriResponse {
            uri: "uri1".to_string(),
        };

        let _: Result<Json<UriResponse>, StatusCode> = Ok(Json(expected_response));

        assert!(matches!(result, _));
    }

    #[tokio::test]
    async fn resolve_package_not_found() {
        let mut package_repo = MockPackageRepository::new();

        package_repo
            .expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| Err(RepositoryError::NotFound));

        let result = resolve(Path(("user1".into(), "package1".into())), package_repo).await;

        assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
    }

    #[tokio::test]
    async fn resolve_version_not_found() {
        let mut package_repo = MockPackageRepository::new();

        let package = Package {
            id: "user1/package1".into(),
            name: PackageName::try_from("package1".to_string()).unwrap(),
            user: Username::try_from("user1".to_string()).unwrap(),
            versions: vec![Version {
                name: "1.0.0".into(),
                uri: "uri1".into(),
            }],
        };

        package_repo
            .expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| Ok(package.clone()));

        let result = resolve(
            Path(("user1".into(), "package1@2.0.0".into())),
            package_repo,
        )
        .await;

        assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
    }

    #[tokio::test]
    async fn invalid_package_name_returns_bad_request1() {
        let mut package_repo = MockPackageRepository::new();

        package_repo
            .expect_read()
            .times(0);

        let result = resolve(
            Path(("user1".into(), "pack!age1@1.0.0".into())),
            package_repo,
        )
        .await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }

    #[tokio::test]
    async fn invalid_package_name_returns_bad_request2() {
        let mut package_repo = MockPackageRepository::new();

        package_repo
            .expect_read()
            .times(0);

        let result = resolve(
            Path(("user1".into(), "pack age1@1.0.0".into())),
            package_repo,
        )
        .await;

        assert!(matches!(result, Err(StatusCode::BAD_REQUEST)));
    }
}

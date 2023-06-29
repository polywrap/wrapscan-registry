use axum::{extract::Path, http::StatusCode, response::{Response}, body::BoxBody};

use crate::{Package, Repository, extract_package_and_version, resolving::ResolveError, resolve_package};

const WRAP_URI_HEADER: &str = "x-wrap-uri";

pub async fn resolve(
    Path((user, package_and_version)): Path<(String, String)>,
    package_repo: impl Repository<Package>
) -> Result<Response, StatusCode> {
    let (package_name, version_name) = extract_package_and_version(&package_and_version);

    let uri = resolve_package(&user, package_name, version_name, &package_repo).await
        .map_err(|e| match e {
            ResolveError::PackageNotFound => StatusCode::NOT_FOUND,
            ResolveError::VersionNotFound => StatusCode::NOT_FOUND,
            ResolveError::RepositoryError(e) => {
                eprintln!("INTERNAL_SERVER_ERROR resolving package: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            },
        })?;
   
    let response: Response = Response::builder()
        .status(StatusCode::OK)
        .header(WRAP_URI_HEADER, uri)
        .body(BoxBody::default())
        .unwrap();

    Ok(response)
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::{http::StatusCode, Json, extract::Path};
    use mockall::{mock, predicate::eq};

    use crate::{Package, Repository, RepositoryError, functions::resolve, Version, UriResponse};

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
      let mut package_repo = MockPackageRepository::new();

        let uri = "uri1";
        let package = Package {
            id: "user1/package1".into(),
            name: "package1".into(),
            user: "user1".into(),
            versions: vec![Version { name: "1.0.0".into(), uri: uri.to_string() }]
        };

        package_repo.expect_read()
            .with(eq("user1/package1".to_string()))
                .return_once(move |_| {
                    Ok(package.clone())
                });

        let result = resolve(Path(("user1".into(), "package1@1.0.0".into())), package_repo).await;

        let expected_response = UriResponse { uri: uri.to_string() };

        let _: Result<Json<UriResponse>, StatusCode> =  Ok(Json(expected_response));

        assert!(matches!(result, _));
    }

    #[tokio::test]
    async fn resolve_package_not_found() {
        let mut package_repo = MockPackageRepository::new();

        package_repo.expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| {
                Err(RepositoryError::NotFound)
            });

        let result = resolve(Path(("user1".into(), "package1".into())), package_repo).await;

        assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
    }

    #[tokio::test]
    async fn resolve_version_not_found() {
        let mut package_repo = MockPackageRepository::new();

        let package = Package {
            id: "user1/package1".into(),
            name: "package1".into(),
            user: "user1".into(),
            versions: vec![Version { name: "1.0.0".into(), uri: "uri1".into() }]
        };

        package_repo.expect_read()
            .with(eq("user1/package1".to_string()))
              .return_once(move |_| {
                  Ok(package.clone())
              });

        let result = resolve(Path(("user1".into(), "package1@2.0.0".into())), package_repo).await;

        assert!(matches!(result, Err(StatusCode::NOT_FOUND)));
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::{http::StatusCode, Json, extract::Path};
    use mockall::{mock, predicate::eq};
    use gateway_service::{Version, Package, UriResponse, RepositoryError, Repository, resolve};

    mock! {
      PackageRepository {} 
        #[async_trait]
        impl Repository<Package> for PackageRepository { 
            async fn read(&self, key: &str) -> Result<Package, RepositoryError>;
            async fn update(&self, entity: &Package) -> Result<(), RepositoryError>;
        }
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

    #[tokio::test]
    async fn resolve_successful() {
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

        let result = gateway_service::resolve(Path(("user1".into(), "package1@1.0.0".into())), package_repo).await;

        let expected_response = UriResponse { uri: uri.to_string() };

        let _: Result<Json<UriResponse>, StatusCode> =  Ok(Json(expected_response));

        assert!(matches!(result, _));
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use axum::{http::StatusCode, Json, extract::Path};
    use mockall::{mock, predicate::eq};
    use gateway_service::{Version, Package, RepositoryError, Repository, publish, UriBody};

    mock! {
      PackageRepository {} 
        #[async_trait]
        impl Repository<Package> for PackageRepository { 
            async fn read(&self, key: &str) -> Result<Package, RepositoryError>;
            async fn update(&self, entity: &Package) -> Result<(), RepositoryError>;
        }
    }

    #[tokio::test]
    async fn publish_version() {
        let package = Package {
            id: "user1/package1".into(),
            name: "package1".into(),
            user: "user1".into(),
            versions: vec![Version { name: "1.0.0".into(), uri: "uri1".into() }]
        };

        let new_version = Version { name: "2.0.0".into(), uri: "uri2".into() };

        let mut package_repo = MockPackageRepository::new();

        let read_package = package.clone();
        package_repo.expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| {
                Ok(read_package.clone())
            });

        let package = package.clone();
        package_repo.expect_update()
            .withf(move |p| {
                &p.id == &package.id && p.versions.len() == 2 && p.versions[1] == new_version
            })
            .return_once(move |_| {
                Ok(())
            });

        let body: Json<UriBody> = Json(UriBody { uri: "uri2".into() });
        let result = publish(Path(("user1".into(), "package1@2.0.0".into())), body, package_repo).await.unwrap();

        assert!(matches!(result.status(), StatusCode::OK));
    }
}

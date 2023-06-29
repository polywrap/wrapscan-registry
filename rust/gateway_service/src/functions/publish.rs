use axum::{body::BoxBody, extract::Path, http::StatusCode, response::Response, Json};

use crate::{
    extract_package_and_version, publishing::publish_package, routes::UriBody, Package, Repository, username::Username, package_name::PackageName,
};

pub async fn publish(
    Path((user, package_and_version)): Path<(String, String)>,
    Json(body): Json<UriBody>,
    package_repo: impl Repository<Package>,
) -> Result<Response, StatusCode> {
    let (username, package_name, version_name) = build_username_package_and_version(user, &package_and_version)?;

    let uri = body.uri;

    publish_package(&username, &package_name, version_name, uri, package_repo)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = Response::builder()
        .status(StatusCode::OK)
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

    use crate::{
        functions::publish, routes::UriBody, Package, Repository, RepositoryError, Version, username::Username, package_name::PackageName,
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
    async fn publish_version() {
        let package = Package {
            id: "user1/package1".into(),
            name: PackageName::try_from("package1".to_string()).unwrap(),
            user: Username::try_from("user1".to_string()).unwrap(),
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

        let read_package = package.clone();
        package_repo
            .expect_read()
            .with(eq("user1/package1".to_string()))
            .return_once(move |_| Ok(read_package.clone()));

        let package = package.clone();
        package_repo
            .expect_update()
            .withf(move |p| {
                &p.id == &package.id && p.versions.len() == 2 && p.versions[1] == new_version
            })
            .return_once(move |_| Ok(()));

        let body: Json<UriBody> = Json(UriBody { uri: "uri2".into() });
        let result = publish(
            Path(("user1".into(), "package1@2.0.0".into())),
            body,
            package_repo,
        )
        .await
        .unwrap();

        assert!(matches!(result.status(), StatusCode::OK));
    }
}

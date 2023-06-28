use crate::{Package, Repository, Version};

use super::error::PublishError;

pub async fn publish_latest_version(
    package: &mut Package, 
    uri: String, 
    package_repo: impl Repository<Package>
) -> Result<(), PublishError> {
    if package.versions.len() > 1 {
        return Err(PublishError::LatestVersionNotAllowed);
    }

    if package.versions.len() == 1 {
        let existing_version = &mut package.versions[0];

        if existing_version.name != "latest" {
            return Err(PublishError::LatestVersionNotAllowed);
        }

        existing_version.uri = uri;
    } else {
        package.versions.push(Version {
            name: "latest".to_string(),
            uri: uri.clone(),
        });
    }

    package_repo.update(&package).await
        .map_err(|e| PublishError::RepositoryError(e.to_string()))?;

    Ok(())
}

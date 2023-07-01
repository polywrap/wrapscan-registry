use reqwest::StatusCode;

use crate::{extract_package_and_version, package_name::PackageName, username::Username};

pub fn get_username_package_and_version(
    username: String,
    package_and_version: &str,
) -> Result<(Username, PackageName, Option<&str>), StatusCode> {
    let username = username.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    let (package_name, version_name) = extract_package_and_version(package_and_version);

    let package_name = package_name.parse().map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok((username, package_name, version_name))
}

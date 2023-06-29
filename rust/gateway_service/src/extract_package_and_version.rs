/// # Extracts the package name and version from a string.
pub fn extract_package_and_version(package_and_version: &str) -> (&str, Option<&str>) {
    let (package_name, version_name) = package_and_version.split_at(
        package_and_version
            .find('@')
            .unwrap_or(package_and_version.len()),
    );

    let version_name = version_name.strip_prefix('@').filter(|s| !s.is_empty());

    (package_name, version_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_package_and_version() {
        assert_eq!(
            extract_package_and_version("name@version"),
            ("name", Some("version"))
        );
        assert_eq!(extract_package_and_version("name"), ("name", None));
        assert_eq!(extract_package_and_version("name@"), ("name", None));
    }
}

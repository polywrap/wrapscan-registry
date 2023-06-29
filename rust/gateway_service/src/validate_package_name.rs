use regex::Regex;

pub fn validate_package_name(package_name: &str) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9_-]*$").unwrap();
    re.is_match(package_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_package_name() {
        assert_eq!(validate_package_name("test"), true);
        assert_eq!(validate_package_name("test-123"), true);
        assert_eq!(validate_package_name("test_123"), true);
        assert_eq!(validate_package_name("TEST-123_abc"), true);
        assert_eq!(validate_package_name("test-123_abc-xyz"), true);
        assert_eq!(validate_package_name("test-123_abc-xyz-"), true);
        assert_eq!(validate_package_name("test-123_abc-xyz+"), false);
        assert_eq!(validate_package_name("test/a"), false);
        assert_eq!(validate_package_name("test@a"), false);
    }
}

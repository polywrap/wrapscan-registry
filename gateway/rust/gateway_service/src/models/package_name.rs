use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use regex::Regex;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PackageName(String);

impl Display for PackageName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct PackageNameParseError;

impl Display for PackageNameParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid package name")
    }
}

impl std::error::Error for PackageNameParseError {}

impl FromStr for PackageName {
    type Err = &'static PackageNameParseError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        if name.len() < 3 || name.len() > 50 {
            return Err(&PackageNameParseError);
        }

        let re = Regex::new(r"^[a-zA-Z0-9_-]*$").unwrap();

        if !re.is_match(name) {
            return Err(&PackageNameParseError);
        }

        Ok(Self(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_package_name() {
        assert!("te".parse::<PackageName>().is_err());
        assert!("t1234567890123456789".parse::<PackageName>().is_ok());
        assert!("t12345678901234567890123456789012345678901234567890"
            .parse::<PackageName>()
            .is_err());
        assert!("test".parse::<PackageName>().is_ok());
        assert!("test-123".parse::<PackageName>().is_ok());
        assert!("test_123".parse::<PackageName>().is_ok());
        assert!("TEST-123_abc".parse::<PackageName>().is_ok());
        assert!("test-123_abc-xyz".parse::<PackageName>().is_ok());
        assert!("test-123_abc-xyz-".parse::<PackageName>().is_ok());
        assert!("test-123_abc-xyz+".parse::<PackageName>().is_err());
        assert!("test/a".parse::<PackageName>().is_err());
        assert!("test@a".parse::<PackageName>().is_err());
    }
}

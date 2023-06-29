use std::fmt::{Display, Formatter, self};

use regex::Regex;
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PackageName(String);

impl PackageName {
    pub fn try_new(name: String) -> Option<Self> {
        if name.len() < 3 || name.len() > 20 {
            return None;
        }

        let re = Regex::new(r"^[a-zA-Z0-9_-]*$").unwrap();
       
        if !re.is_match(&name) {
            return None;
        }

        Some(Self(name))
    }
}

impl Display for PackageName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct PackageNameParseError;

impl TryFrom<String> for PackageName {
    type Error = &'static PackageNameParseError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        if let Some(name) = Self::try_new(name) {
            Ok(name)
        } else {
            Err(&PackageNameParseError {})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_package_name() {
        assert_eq!(PackageName::try_from("test".to_string()).is_ok(), true);
        assert_eq!(PackageName::try_from("test-123".to_string()).is_ok(), true);
        assert_eq!(PackageName::try_from("test_123".to_string()).is_ok(), true);
        assert_eq!(PackageName::try_from("TEST-123_abc".to_string()).is_ok(), true);
        assert_eq!(PackageName::try_from("test-123_abc-xyz".to_string()).is_ok(), true);
        assert_eq!(PackageName::try_from("test-123_abc-xyz-".to_string()).is_ok(), true);
        assert_eq!(PackageName::try_from("test-123_abc-xyz+".to_string()).is_ok(), false);
        assert_eq!(PackageName::try_from("test/a".to_string()).is_ok(), false);
        assert_eq!(PackageName::try_from("test@a".to_string()).is_ok(), false);
    }
}


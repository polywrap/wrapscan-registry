use std::{fmt::{Display, Formatter, self}, str::FromStr};

use regex::Regex;
use serde_derive::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PackageName(String);

impl Display for PackageName {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct PackageNameParseError;

impl FromStr for PackageName {
    type Err = &'static PackageNameParseError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        if name.len() < 3 || name.len() > 20 {
            return Err(&PackageNameParseError {});
        }

        let re = Regex::new(r"^[a-zA-Z0-9_-]*$").unwrap();
       
        if !re.is_match(&name) {
            return Err(&PackageNameParseError {});
        }
        
        Ok(Self(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_package_name() {
        assert_eq!("test".parse::<PackageName>().is_ok(), true);
        assert_eq!("test-123".parse::<PackageName>().is_ok(), true);
        assert_eq!("test_123".parse::<PackageName>().is_ok(), true);
        assert_eq!("TEST-123_abc".parse::<PackageName>().is_ok(), true);
        assert_eq!("test-123_abc-xyz".parse::<PackageName>().is_ok(), true);
        assert_eq!("test-123_abc-xyz-".parse::<PackageName>().is_ok(), true);
        assert_eq!("test-123_abc-xyz+".parse::<PackageName>().is_ok(), false);
        assert_eq!("test/a".parse::<PackageName>().is_ok(), false);
        assert_eq!("test@a".parse::<PackageName>().is_ok(), false);
    }
}


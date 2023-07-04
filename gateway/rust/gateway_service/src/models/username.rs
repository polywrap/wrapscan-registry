use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use regex::Regex;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Username(String);

impl Display for Username {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct UsernameParseError;

impl Display for UsernameParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid username")
    }
}

impl std::error::Error for UsernameParseError {}

impl FromStr for Username {
    type Err = &'static UsernameParseError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        if name.len() < 3 || name.len() > 50 {
            return Err(&UsernameParseError);
        }

        let re = Regex::new(r"^[a-zA-Z0-9_]*$").unwrap();

        if !re.is_match(name) {
            return Err(&UsernameParseError);
        }

        Ok(Self(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_username() {
        assert!("te".parse::<Username>().is_err());
        assert!("t1234567890123456789".parse::<Username>().is_ok());
        assert!("t12345678901234567890".parse::<Username>().is_ok());
        assert!("t12345678901234567890123456789012345678901234567890"
            .parse::<Username>()
            .is_err());
        assert!("test".parse::<Username>().is_ok());
        assert!("test-123".parse::<Username>().is_err());
        assert!("test_123".parse::<Username>().is_ok());
        assert!("TEST-123_abc".parse::<Username>().is_err());
        assert!("test-123_abc-xyz".parse::<Username>().is_err());
        assert!("test-123_abc-xyz-".parse::<Username>().is_err());
        assert!("test-123_abc-xyz+".parse::<Username>().is_err());
        assert!("test/a".parse::<Username>().is_err());
        assert!("test@a".parse::<Username>().is_err());
    }
}

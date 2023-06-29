use std::{fmt::{Display, Formatter, self}, str::FromStr};

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

impl FromStr for Username {
    type Err = &'static UsernameParseError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        if name.len() < 3 || name.len() > 20 {
            return Err(&UsernameParseError {});
        }

        let re = Regex::new(r"^[a-zA-Z0-9_]*$").unwrap();
       
        if !re.is_match(&name) {
            return Err(&UsernameParseError {});
        }
        
        Ok(Self(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_user_name() {
        assert_eq!("test".parse::<Username>().is_ok(), true);
        assert_eq!("test-123".parse::<Username>().is_ok(), false);
        assert_eq!("test_123".parse::<Username>().is_ok(), true);
        assert_eq!("TEST-123_abc".parse::<Username>().is_ok(), false);
        assert_eq!("test-123_abc-xyz".parse::<Username>().is_ok(), false);
        assert_eq!("test-123_abc-xyz-".parse::<Username>().is_ok(), false);
        assert_eq!("test-123_abc-xyz+".parse::<Username>().is_ok(), false);
        assert_eq!("test/a".parse::<Username>().is_ok(), false);
        assert_eq!("test@a".parse::<Username>().is_ok(), false);
    }
}

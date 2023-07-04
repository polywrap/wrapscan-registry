use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use polywrap_core::uri::Uri;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq)]
pub struct WrapUri(Uri);

impl Serialize for WrapUri {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for WrapUri {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let uri = String::deserialize(deserializer)?;
        Uri::try_from(uri.as_str())
            .map_err(serde::de::Error::custom)
            .map(WrapUri)
    }
}

impl Display for WrapUri {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct UriParseError;

impl FromStr for WrapUri {
    type Err = &'static UriParseError;

    fn from_str(uri: &str) -> Result<Self, Self::Err> {
        Uri::try_from(uri).map_err(|_| &UriParseError).map(WrapUri)
    }
}

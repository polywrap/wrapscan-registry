use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{package_name::PackageName, username::Username, IVersion, WrapUri};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Package {
    pub id: String,
    pub user: Username,
    pub name: PackageName,
    pub versions: Vec<Version>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Version {
    pub name: String,
    pub uri: WrapUri,
}

impl IVersion for Version {
    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PartialVersion {
    pub name: String,
}

impl IVersion for PartialVersion {
    fn name(&self) -> String {
        self.name.clone()
    }
}

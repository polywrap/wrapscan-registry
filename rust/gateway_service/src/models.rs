use serde::{Deserialize, Serialize};

use crate::{IVersion, username::Username, package_name::PackageName};

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
    pub uri: String,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct UriResponse {
    pub uri: String,
}

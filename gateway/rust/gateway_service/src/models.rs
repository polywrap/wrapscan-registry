use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::{package_name::PackageName, username::Username, IVersion, WrapUri};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Package {
    pub id: String,
    pub name: PackageName,
    pub user: Username,
    pub versions: Vec<Version>,
    pub created_on: u128,
}

impl PartialEq for Package {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Package {
    pub fn new(name: PackageName, user: Username) -> Self {
        let id = format!("{}/{}", user, name);

        let created_on = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        Self {
            id,
            user,
            name,
            versions: vec![],
            created_on,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Version {
    pub name: String,
    pub uri: WrapUri,
    pub created_on: u128,
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.uri == other.uri
    }
}

impl Version {
    pub fn new(name: String, uri: WrapUri) -> Self {
        let created_on = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        Self {
            name,
            uri,
            created_on,
        }
    }
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

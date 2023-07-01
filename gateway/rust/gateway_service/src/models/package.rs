use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use super::{PackageName, Username, Version};

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

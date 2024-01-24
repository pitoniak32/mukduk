use std::{fmt::Display, path::PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub path: PathBuf,
    pub name: String,
}

impl Project {
    pub fn new(path: PathBuf, name: String) -> Self {
        Self {
            path,
            name: name.replace('.', "_"),
        }
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

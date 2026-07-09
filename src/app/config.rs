use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub recents: Vec<PathBuf>,
    pub statement_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            recents: Vec::new(),
            statement_path: None,
        }
    }
}

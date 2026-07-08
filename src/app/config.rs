use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub statement_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            statement_path: None,
        }
    }
}

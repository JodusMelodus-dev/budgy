use std::path::PathBuf;

use egui::Color32;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub recents: Vec<PathBuf>,
    pub statement_path: Option<PathBuf>,
    pub categories: Vec<(String, Color32)>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            recents: Vec::new(),
            statement_path: None,

            categories: vec![
                ("Salary".to_string(), Color32::from_hex("#5B643E").unwrap()),
                (
                    "Pocket Money".to_string(),
                    Color32::from_hex("#314350").unwrap(),
                ),
                ("Mobile".to_string(), Color32::from_hex("#C89839").unwrap()),
                ("Clothes".to_string(), Color32::from_hex("#AB5E16").unwrap()),
                (
                    "Personal Care".to_string(),
                    Color32::from_hex("#7B533B").unwrap(),
                ),
                (
                    "Goodwill".to_string(),
                    Color32::from_hex("#792318").unwrap(),
                ),
                ("Fuel".to_string(), Color32::from_hex("#44724F").unwrap()),
                ("Food".to_string(), Color32::from_hex("#4B8299").unwrap()),
                ("Bank".to_string(), Color32::from_hex("#99BFDF").unwrap()),
            ],
        }
    }
}

use std::fs::File;

use egui::Ui;
use polars::io::{SerWriter, csv::write::CsvWriter};
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageLevel};

use crate::app::Budgy;

impl Budgy {
    pub fn import_csv(&mut self) {
        self.config.statement_path = FileDialog::new()
            .add_filter("CSV Files", &["csv"])
            .pick_file();

        if let Some(path) = &self.config.statement_path {
            if !self.config.recent.contains(path) {
                self.config.recent.insert(0, path.to_path_buf());
            }
        }
    }

    pub fn export_csv(&mut self) {
        let path = FileDialog::new()
            .set_file_name("budget summary.csv")
            .add_filter("CSV Files", &["csv"])
            .save_file()
            .expect("Failed to get save path");

        let file = File::create(&path).expect("Failed to create file");
        if let Some(mut budget_summary_df) = self.budget_summary.clone() {
            CsvWriter::new(file)
                .include_header(true)
                .with_separator(b',')
                .finish(&mut budget_summary_df)
                .expect("Failed to export file");

            MessageDialog::new()
                .set_level(MessageLevel::Info)
                .set_description(format!(
                    "Successfully exported budget summary to: {}",
                    path.to_str().unwrap()
                ))
                .set_buttons(MessageButtons::Ok)
                .show();
        }
    }

    pub fn open_recent(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            for path in &self.config.recent {
                if ui
                    .button(path.file_name().unwrap().to_str().unwrap_or("NULL"))
                    .clicked()
                {
                    self.config.statement_path = Some(path.to_path_buf());
                }
            }
        });
    }
}

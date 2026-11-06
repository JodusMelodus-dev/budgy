use egui::vec2;

use crate::app::{Budgy, file::BudgyFileType};

impl Budgy {
    pub fn display_settings(&mut self, ui: &mut egui::Ui) {
        ui.spacing_mut().button_padding = vec2(8.0, 2.0);

        ui.heading("Settings");
        ui.separator();

        ui.heading("Budget");
        if ui.button("Import Budget").clicked() {
            self.open_csv_file(BudgyFileType::Budget);
        }
    }
}

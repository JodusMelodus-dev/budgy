use egui::{Align, Layout, vec2};

use crate::app::{Budgy, BudgyError, file::BudgyFileType};

impl Budgy {
    pub fn display_settings(&self, ui: &mut egui::Ui) -> Result<(), BudgyError> {
        ui.spacing_mut().button_padding = vec2(18.0, 6.0);

        ui.heading("Settings");
        ui.separator();

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading("Import Budget");
                ui.label("Import your existing budget from a .csv file");
            });
            ui.with_layout(Layout::right_to_left(Align::RIGHT), |ui| {
                if ui.button("Import").clicked() {
                    self.open_csv_file(BudgyFileType::Budget);
                }
            });
        });

        Ok(())
    }
}

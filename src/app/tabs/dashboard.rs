use crate::app::{Budgy, BudgyError};

impl Budgy {
    pub fn display_dashboard(&self, ui: &mut egui::Ui) -> Result<(), BudgyError> {
        ui.heading("Dashboard");
        ui.separator();
        Ok(())
    }
}

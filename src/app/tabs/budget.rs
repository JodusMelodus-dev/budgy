use crate::app::Budgy;

impl Budgy {
    pub fn display_budget_tab(&self, ui: &mut egui::Ui) {
        ui.heading("Budget");
    }
}

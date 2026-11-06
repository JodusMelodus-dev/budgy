use crate::app::Budgy;

impl Budgy {
    pub fn display_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.heading("Dashboard");
        ui.separator();
    }
}

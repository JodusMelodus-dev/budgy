use crate::app::{Budgy, help::load_budget};

impl Budgy {
    pub fn display_budget_tab(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |_ui| {});

        ui.separator();

        if let Some(df) = &self.budget_df {
            self.display_dataframe(ui, df);
        } else if let Some(lf) = self.budget_lf.clone() {
            self.budget_df = Some(lf.collect().expect("Failed to collect budget lazy frame"));
        } else {
            self.budget_lf = load_budget();
        }
    }
}

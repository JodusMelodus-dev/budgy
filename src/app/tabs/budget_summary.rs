use crate::app::Budgy;

impl Budgy {
    pub fn display_budget_summary_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(df) = &self.budget_summary {
            self.display_dataframe(ui, &df);
        }
    }
}

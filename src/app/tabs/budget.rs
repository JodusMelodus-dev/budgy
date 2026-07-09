use crate::app::Budgy;

impl Budgy {
    pub fn display_budget_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(budget) = self.budget_lf.clone() {
            self.budget_df = Some(budget.collect().expect("Failed to collect budget"));

            if let Some(budget) = &self.budget_df {
                self.display_dataframe(ui, &budget);
            }
        }
    }
}

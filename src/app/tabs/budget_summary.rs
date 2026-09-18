use polars::lazy::dsl::col;

use crate::app::Budgy;

impl Budgy {
    pub fn display_budget_summary_tab(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |_ui| {});

        ui.separator();

        if let Some(df) = &self.budget_summary {
            self.display_dataframe(ui, df);
        } else {
            if let Some(statement) = self.statement_lf.clone() {
                if let Some(budget) = self.budget_lf.clone() {
                    let mut result = statement
                        .group_by([col("Parent Category")])
                        .agg([
                            col("Money In").sum(),
                            col("Money Out").sum(),
                            col("Fee").sum(),
                        ])
                        .left_join(budget, col("Parent Category"), col("Category"));

                    result = result.with_column(
                        (col("Money In") + col("Money Out") + col("Fee") - col("Budget Amount"))
                            .alias("Balance"),
                    );

                    self.budget_summary = result.collect().ok();
                }
            } else {
                ui.heading("Missing budget");
            }
        }
    }
}

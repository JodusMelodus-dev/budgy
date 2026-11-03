use polars::{chunked_array::ops::SortMultipleOptions, lazy::dsl::col};

use crate::app::Budgy;

impl Budgy {
    pub fn display_budget_summary_tab(&mut self, ui: &mut egui::Ui) {
        let Some(df) = &mut self.budget_summary else {
            let Some(mut summary) = self.previous_summary_lf.clone() else {
                log::error!("Missing previous summary");
                return;
            };

            let Some(statement) = self.statement_lf.clone() else {
                log::error!("Missing statement");
                return;
            };

            let Some(budget) = self.budget_lf.clone() else {
                log::error!("Missing budget");
                return;
            };

            summary = summary.rename(["Balance"], ["Previous Balance"], true);
            let mut result = summary.left_join(statement, col("Category"), col("Category"));
            result = result
                .group_by([col("Category")])
                .agg([
                    col("Money In").sum(),
                    col("Money Out").sum(),
                    col("Fee").sum(),
                    col("Previous Balance").max(),
                ])
                .left_join(budget, col("Category"), col("Category"));
            result = result.with_column(
                (col("Money In") + col("Money Out") + col("Fee") - col("Budget Amount")
                    + col("Previous Balance"))
                .alias("Balance"),
            );

            let Ok(res) = result
                .sort(["Category"], SortMultipleOptions::default())
                .collect()
            else {
                log::error!("Failed to generate budget summary");
                return;
            };

            self.budget_summary = Some(res);
            return;
        };

        Self::display_dataframe(ui, df);
    }
}

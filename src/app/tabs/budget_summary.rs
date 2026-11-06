use polars::{chunked_array::ops::SortMultipleOptions, lazy::dsl::col};

use crate::app::{Budgy, BudgyError};

impl Budgy {
    pub fn display_budget_summary_tab(&mut self, ui: &mut egui::Ui) -> Result<(), BudgyError> {
        ui.heading("Summary");
        ui.separator();

        let Some(df) = &mut self.budget_summary else {
            let Some(mut summary) = self.previous_summary_lf.clone() else {
                return Err(BudgyError::Frame("Missing previous summary".to_owned()));
            };

            let Some(statement) = self.statement_lf.clone() else {
                return Err(BudgyError::Frame("Missing statement".to_owned()));
            };

            let Some(budget) = self.budget_lf.clone() else {
                return Err(BudgyError::Frame("Missing budget".to_owned()));
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

            let res = result
                .sort(["Category"], SortMultipleOptions::default())
                .collect()?;
            self.budget_summary = Some(res);
            return Ok(());
        };

        Self::display_dataframe(ui, df);
        Ok(())
    }
}

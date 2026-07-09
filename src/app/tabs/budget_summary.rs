use polars::{
    chunked_array::ops::SortMultipleOptions,
    lazy::dsl::{col, lit},
};

use crate::app::Budgy;

impl Budgy {
    pub fn display_budget_summary_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(statement) = self.statement_lf.clone() {
            let result = statement
                .group_by([col("New Category"), col("Start Date"), col("End Date")])
                .agg([
                    col("Money In").fill_null(lit(0.0)).sum().alias("Total In"),
                    col("Money Out")
                        .fill_null(lit(0.0))
                        .sum()
                        .alias("Total Out"),
                    col("Fee").fill_null(lit(0.0)).sum().alias("Total Fees"),
                    (col("Money In").fill_null(lit(0.0)).sum()
                        + col("Money Out").fill_null(lit(0.0)).sum()
                        + col("Fee").fill_null(lit(0.0)).sum())
                    .alias("Net Total"),
                    (col("Budget Amount") * col("Month Difference"))
                        .max()
                        .alias("Budgetted Amount"),
                    ((col("Money In").fill_null(lit(0.0)).sum()
                        + col("Money Out").fill_null(lit(0.0)).sum()
                        + col("Fee").fill_null(lit(0.0)).sum())
                        - (col("Budget Amount") * col("Month Difference")).max())
                    .alias("NET BUDGET"),
                ])
                .sort(
                    ["Start Date", "New Category"],
                    SortMultipleOptions::new().with_order_descending(false),
                );

            let final_result = result
                .group_by(["New Category"])
                .agg([(col("NET BUDGET").sum()).alias("Budget Balance")])
                .sort(
                    ["Budget Balance"],
                    SortMultipleOptions::new().with_order_descending(false),
                );

            self.budget_summary = Some(
                final_result
                    .collect()
                    .expect("Failed to generate budget summary"),
            );

            if let Some(budget_summary) = &self.budget_summary {
                self.display_dataframe(ui, budget_summary);
            }
        }
    }
}

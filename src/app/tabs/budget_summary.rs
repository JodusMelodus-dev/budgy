use polars::{
    chunked_array::ops::SortMultipleOptions,
    lazy::dsl::{col, lit},
};

use crate::app::Budgy;

impl Budgy {
    pub fn display_budget_summary_tab(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |_ui| {});

        ui.separator();

        if let Some(df) = &self.budget_summary {
            self.display_dataframe(ui, df);
        } else {
            if let Some(mut statement) = self.statement_lf.clone() {
                if let Some(budget) = self.budget_lf.clone() {
                    statement = statement
                        .left_join(budget, col("Category"), col("Category"))
                        .filter(
                            col("Posting Date")
                                .gt_eq(col("Start Date"))
                                .and(col("Posting Date").lt_eq("End Date")),
                        );

                    let result = statement
                        .group_by([col("Category"), col("Start Date"), col("End Date")])
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
                            ["Start Date", "Category"],
                            SortMultipleOptions::new().with_order_descending(false),
                        );

                    let final_result = result
                        .group_by(["Category"])
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
                }
            } else {
                ui.heading("Missing budget");
            }
        }
    }
}

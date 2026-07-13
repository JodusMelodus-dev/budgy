use polars::{
    chunked_array::ops::SortMultipleOptions,
    frame::UniqueKeepStrategy,
    lazy::dsl::{col, lit},
    prelude::JoinType,
};

use crate::app::Budgy;

impl Budgy {
    pub fn display_budget_summary_tab(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |_ui| {});

        ui.separator();

        if let Some(df) = &self.budget_summary {
            self.display_dataframe(ui, df);
        } else {
            if let Some(statement) = self.statement_lf.clone() {
                if let Some(lookup) = self.lookup_lf.clone() {
                    if let Some(budget) = self.budget_lf.clone() {
                        let combined = statement
                            .clone()
                            .with_column(lit(1).alias("Join Key"))
                            .join(
                                lookup,
                                [col("Join Key")],
                                [col("Join Key")],
                                JoinType::Inner.into(),
                            );
                        let matched =
                            combined.filter(col("Description").str().contains(col("Mask"), true));
                        let stmt = matched
                            .drop([col("Join Key")])
                            .unique(Some(vec!["Nr".to_string()]), UniqueKeepStrategy::First);

                        self.statement_lf = Some(
                            stmt.filter(col("New Category").is_not_null())
                                .left_join(budget, col("New Category"), col("Category"))
                                .filter(
                                    col("Posting Date")
                                        .gt_eq(col("Start Date"))
                                        .and(col("Posting Date").lt_eq("End Date")),
                                ),
                        );

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
                        }
                    } else {
                        ui.heading("Missing budget");
                    }
                } else {
                    ui.heading("Missing lookup");
                }
            } else {
                ui.heading("Open a statement to get started");
            }
        }
    }
}

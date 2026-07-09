use help::{generate_undefined_categories, load_statement};
use polars::{
    chunked_array::ops::SortMultipleOptions,
    frame::UniqueKeepStrategy,
    lazy::{
        dsl::{col, lit},
        frame::IntoLazy,
    },
    prelude::JoinType,
};

use crate::app::{Budgy, help};

impl Budgy {
    pub fn display_statements_tab(&mut self, _ui: &mut egui::Ui) {
        if self.budget_summary.is_none() {
            if let Some(path) = &self.config.statement_path {
                self.statement_lf = load_statement(path);

                if let Some(lookup) = self.lookup_lf.clone() {
                    if let Some(statement) = self.statement_lf.clone() {
                        if let Some(budget) = self.budget_lf.clone() {
                            let combined = statement.join(
                                lookup,
                                [col("Join Key")],
                                [col("Join Key")],
                                JoinType::Inner.into(),
                            );
                            let matched = combined
                                .filter(col("Description").str().contains(col("Mask"), true));
                            let stmt = matched
                                .drop([col("Join Key")])
                                .unique(Some(vec!["Nr".to_string()]), UniqueKeepStrategy::First);

                            let summary = stmt
                                .clone()
                                .lazy()
                                .filter(col("New Category").is_not_null())
                                .left_join(budget, col("New Category"), col("Category"))
                                .filter(
                                    col("Date")
                                        .gt_eq(col("Start Date"))
                                        .and(col("Date").lt_eq("End Date")),
                                )
                                .collect()
                                .expect("Failed to collect summray");

                            let result = summary
                                .clone()
                                .lazy()
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

                            generate_undefined_categories(stmt)
                                .expect("Failed to get undef categories");

                            self.budget_summary = Some(
                                final_result
                                    .collect()
                                    .expect("Failed to generate budget summary"),
                            );
                        }
                    }
                }
            }
        }
    }
}

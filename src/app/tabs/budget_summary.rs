use polars::{
    chunked_array::ops::SortMultipleOptions,
    functions::concat_df_diagonal,
    lazy::{
        dsl::{col, lit, when},
        frame::{IntoLazy, LazyFrame},
    },
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
                    statement = Self::add_starting_budget(statement.clone(), budget.clone());

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
                                .alias("Budgeted Amount"),
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

    fn add_starting_budget(statement: LazyFrame, budget: LazyFrame) -> LazyFrame {
        let first_transaction = statement.clone().limit(1).drop([
            "Nr",
            "Category",
            "Transaction Date",
            "Description",
            "Original Description",
            "Parent Category",
        ]);
        let starting_balance = first_transaction.with_column(
            (col("Balance")
                - when(col("Money In").is_not_null())
                    .then(col("Money In"))
                    .when(col("Money Out").is_not_null())
                    .then(col("Money Out"))
                    .when(col("Fee").is_not_null())
                    .then(col("Fee"))
                    .otherwise(lit(0.0)))
            .alias("Balance"),
        );

        let starting_balance_with_budget = starting_balance
            .cross_join(budget, None)
            .filter(
                col("Posting Date")
                    .lt_eq(col("End Date"))
                    .and(col("Posting Date").gt_eq(col("Start Date"))),
            )
            .drop([
                "B_Nr",
                "Color",
                "Month Difference",
                "Money In",
                "Money Out",
                "Fee",
                "Start Date",
                "End Date",
            ]);

        let ratios = starting_balance_with_budget
            .with_column((-col("Budget Amount") / col("Budget Amount").max()).alias("Ratio"))
            .filter(col("Budget Amount").neq(col("Budget Amount").max()));

        let starting = ratios
            .with_column((col("Balance") * col("Ratio")).alias("Money In"))
            .drop(["Balance", "Budget Amount", "Ratio"]);

        let statement =
            concat_df_diagonal(&[statement.collect().unwrap(), starting.collect().unwrap()])
                .unwrap()
                .lazy()
                .drop(["Balance"]);

        statement
    }
}

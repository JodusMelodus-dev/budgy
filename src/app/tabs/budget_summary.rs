use polars::{
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
            if let Some(statement) = self.statement_lf.clone() {
                if let Some(budget) = self.budget_lf.clone() {
                    let result = statement
                        .group_by([col("Parent Category")])
                        .agg([
                            col("Money In").sum(),
                            col("Money Out").sum(),
                            col("Fee").sum(),
                        ])
                        .left_join(budget, col("Parent Category"), col("Category"));

                    self.budget_summary = result.collect().ok();
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

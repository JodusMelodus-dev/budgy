use polars::{chunked_array::ops::SortMultipleOptions, lazy::dsl::col};

use crate::app::{Budgy, help::load_previous_summary};

impl Budgy {
    pub fn display_budget_summary_tab(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |_ui| {});

        ui.separator();

        if let Some(df) = &self.budget_summary {
            self.display_dataframe(ui, df);
        } else {
            if let Some(statement) = self.statement_lf.clone() {
                if let Some(budget) = self.budget_lf.clone() {
                    if let Some(mut summary) = self.previous_summary_lf.clone() {
                        summary = summary
                            .with_column(col("Balance").alias("Previous Balance"));

                        let result = statement
                            .group_by([col("Parent Category")])
                            .agg([
                                col("Money In").sum(),
                                col("Money Out").sum(),
                                col("Fee").sum(),
                            ])
                            .left_join(budget, col("Parent Category"), col("Category"));

                        let mut new = result.left_join(
                            summary,
                            col("Parent Category"),
                            col("Parent Category"),
                        );

                        new = new.with_column(
                            (col("Money In") + col("Money Out") + col("Fee")
                                - col("Budget Amount")
                                + col("Previous Balance"))
                            .alias("Balance"),
                        );

                        self.budget_summary = new
                            .sort(["Parent Category"], SortMultipleOptions::default())
                            .collect()
                            .ok();
                    } else {
                        self.previous_summary_lf = load_previous_summary();
                    }
                } else {
                    println!("No budget");
                }
            } else {
                ui.heading("Missing budget");
            }
        }
    }
}

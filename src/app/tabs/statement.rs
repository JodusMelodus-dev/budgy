use help::load_statement;
use polars::{frame::UniqueKeepStrategy, lazy::dsl::col, prelude::JoinType};

use crate::app::{Budgy, help};

impl Budgy {
    pub fn display_statements_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(df) = &self.statement_df {
            self.display_dataframe(ui, df);
        } else if let Some(lf) = self.statement_lf.clone() {
            self.statement_df = Some(
                lf.collect()
                    .expect("Failed to collect statement lazy frame"),
            );
        } else if let Some(path) = &self.config.statement_path {
            if let Some(statement) = load_statement(path) {
                if let Some(lookup) = self.lookup_lf.clone() {
                    if let Some(budget) = self.budget_lf.clone() {
                        let combined = statement.join(
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
                                    col("Date")
                                        .gt_eq(col("Start Date"))
                                        .and(col("Date").lt_eq("End Date")),
                                ),
                        );
                    } else {
                        ui.heading("Missing budget");
                    }
                } else {
                    ui.heading("Missing lookup");
                }
            } else {
                ui.heading("Failed to load statement");
            }
        } else {
            ui.heading("Open a statement to get started");
        }
    }
}

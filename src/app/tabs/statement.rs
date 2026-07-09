use help::load_statement;
use polars::{frame::UniqueKeepStrategy, lazy::dsl::col, prelude::JoinType};

use crate::app::{Budgy, help};

impl Budgy {
    pub fn display_statements_tab(&mut self, ui: &mut egui::Ui) {
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

                            self.statement_lf = Some(
                                stmt.filter(col("New Category").is_not_null())
                                    .left_join(budget, col("New Category"), col("Category"))
                                    .filter(
                                        col("Date")
                                            .gt_eq(col("Start Date"))
                                            .and(col("Date").lt_eq("End Date")),
                                    ),
                            );

                            if let Some(statement) = self.statement_lf.clone() {
                                if let Some(statement) = &self.statement_df {
                                    self.display_dataframe(ui, statement);
                                } else {
                                    self.statement_df = Some(
                                        statement.collect().expect("Failed to collect statement"),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

use polars::{
    datatypes::AnyValue, error::PolarsError, frame::DataFrame, lazy::frame::IntoLazy as _,
    prelude::NamedFrom as _, series::Series,
};
use thiserror::Error;

use crate::app::Budgy;

#[derive(Debug, Error)]
pub enum BudgyBudgetError {
    #[error("Polars failed")]
    Polars(#[from] PolarsError),
}

impl Budgy {
    pub fn display_budget_tab(&mut self, ui: &mut egui::Ui) -> Result<(), BudgyBudgetError> {
        ui.heading("Budget");
        ui.separator();

        if let Some(df) = &self.budget_df {
            let column_names = df.get_column_names();

            egui::ScrollArea::horizontal()
                .id_salt("budget")
                .show(ui, |ui| {
                    let table = egui_extras::TableBuilder::new(ui).striped(true);
                    let height = df.height();

                    table
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .columns(egui_extras::Column::auto(), column_names.len())
                        .header(20.0, |mut header| {
                            for name in &column_names[..column_names.len()] {
                                header.col(|ui| {
                                    ui.strong(name.to_string());
                                });
                            }
                        })
                        .body(|body| {
                            body.rows(22.0, height, |mut row| {
                                let row_idx = row.index();
                                for column_name in &column_names {
                                    row.col(|ui| {
                                        if let Ok(column) = df.column(column_name) {
                                            let value =
                                                column.get(row_idx).unwrap_or(AnyValue::Null);
                                            ui.label(format!("{value}"));
                                        }
                                    });
                                }
                            });
                        });
                });
        } else if let Some(lf) = self.budget_lf.clone() {
            self.budget_df = Some(lf.collect().expect("Failed to collect budget lazy frame"));
        } else {
            let category = Series::new("Category".into(), &self.budget.categories);
            let amount = Series::new("Budget Amount".into(), &self.budget.amounts);
            let df = DataFrame::new(vec![category.into(), amount.into()])?;
            self.budget_lf = Some(df.lazy());
        }

        Ok(())
    }
}

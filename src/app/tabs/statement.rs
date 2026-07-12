use std::fs::File;

use help::load_statement;
use polars::{
    datatypes::AnyValue,
    frame::{DataFrame, column::Column},
    io::{SerWriter, csv::write::CsvWriter},
    lazy::{
        dsl::{col, when},
        frame::IntoLazy,
    },
};

use crate::app::{Budgy, help};

const CATEGORIES: [&str; 9] = [
    "Salary",
    "Pocket Money",
    "Mobile",
    "Clothes",
    "Personal Care",
    "Goodwill",
    "Fuel",
    "Food",
    "Bank",
];

impl Budgy {
    pub fn display_statement_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(df) = &self.statement_df {
            let height = df.height();
            let column_names = df.get_column_names();

            egui::ScrollArea::horizontal().show(ui, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .columns(egui_extras::Column::auto(), column_names.len())
                    .header(20.0, |mut header| {
                        for name in &column_names {
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
                                        let value = column.get(row_idx).unwrap_or(AnyValue::Null);

                                        if *column_name == "Category" {
                                            let current_value = self
                                                .statement_deltas
                                                .get(&(row_idx as i64))
                                                .cloned()
                                                .unwrap_or_else(|| {
                                                    if let AnyValue::String(v) = value {
                                                        v.to_string()
                                                    } else {
                                                        "unknown".to_string()
                                                    }
                                                });

                                            ui.menu_button(current_value, |ui| {
                                                for category in CATEGORIES {
                                                    if ui.button(category).clicked() {
                                                        self.statement_deltas.insert(
                                                            row_idx as i64,
                                                            category.to_string(),
                                                        );
                                                    }
                                                }
                                            });
                                        } else {
                                            ui.label(format!("{}", value));
                                        }
                                    }
                                });
                            }
                        });
                    });
            });
        } else if let Some(lf) = self.statement_lf.clone() {
            self.statement_df = Some(
                lf.collect()
                    .expect("Failed to collect statement lazy frame"),
            );
        } else if let Some(path) = &self.config.statement_path {
            self.statement_lf = load_statement(path);
        } else {
            ui.heading("Open a statement to get started");
        }
    }

    pub fn save_updated_categories(&mut self) {
        let rows = self.statement_deltas.keys().cloned().collect::<Vec<i64>>();
        let categories = self
            .statement_deltas
            .values()
            .cloned()
            .collect::<Vec<String>>();

        let row_series = Column::new("Nr".into(), rows);
        let category_series = Column::new("User Picked Category".into(), categories);

        let delta_table = DataFrame::new(vec![row_series, category_series])
            .unwrap()
            .lazy();

        if let Some(statement) = self.statement_lf.clone() {
            let mut updated = statement
                .left_join(delta_table, col("Nr"), col("Nr"))
                .with_column(
                    when(col("User Picked Category").is_not_null())
                        .then(col("User Picked Category"))
                        .otherwise(col("Category"))
                        .alias("Category"),
                )
                .drop(["User Picked Category"])
                .collect()
                .unwrap();

            if let Some(path) = &self.config.statement_path {
                let file = File::create(path).unwrap();
                CsvWriter::new(file)
                    .include_header(true)
                    .with_separator(b',')
                    .finish(&mut updated)
                    .unwrap();
            }
        }
    }
}

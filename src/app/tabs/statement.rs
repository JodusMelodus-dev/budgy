use std::fs::File;

use egui::{Color32, RichText};
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

impl Budgy {
    pub fn display_statement_tab(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if ui.button("Save").clicked() {
                self.save_updated_categories();
            }
        });

        ui.separator();

        if let Some(df) = &self.statement_df {
            let height = df.height();
            let column_names = df.get_column_names();

            egui::ScrollArea::both()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        egui_extras::TableBuilder::new(ui)
                            .striped(true)
                            .vscroll(false)
                            .max_scroll_height(f32::INFINITY)
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

                                    if let AnyValue::Int64(nr) =
                                        df.column(column_names[0]).unwrap().get(row_idx).unwrap()
                                    {
                                        for column_name in &column_names {
                                            row.col(|ui| {
                                                if let Ok(column) = df.column(column_name) {
                                                    let value = column
                                                        .get(row_idx)
                                                        .unwrap_or(AnyValue::Null);

                                                    if *column_name == "Category" {
                                                        let current_value = self
                                                            .statement_deltas
                                                            .get(&nr)
                                                            .cloned()
                                                            .unwrap_or_else(|| {
                                                                if let AnyValue::String(v) = value {
                                                                    v.to_string()
                                                                } else {
                                                                    "unknown".to_string()
                                                                }
                                                            });

                                                        let color = self
                                                            .config
                                                            .categories
                                                            .iter()
                                                            .find(|(k, _)| *k == current_value)
                                                            .unwrap_or(&(
                                                                "".to_string(),
                                                                Color32::DARK_GRAY,
                                                            ))
                                                            .1;

                                                        let text = RichText::new(current_value)
                                                            .color(color);

                                                        let width = ui.available_width()
                                                            - ui.spacing().item_spacing.x;
                                                        egui::ComboBox::from_label("")
                                                            .width(width)
                                                            .selected_text(text.clone())
                                                            .show_ui(ui, |ui| {
                                                                for (category, color) in
                                                                    &self.config.categories
                                                                {
                                                                    let text =
                                                                        RichText::new(category)
                                                                            .color(*color);

                                                                    if ui.button(text).clicked() {
                                                                        self.statement_deltas
                                                                            .insert(
                                                                                nr,
                                                                                category
                                                                                    .to_string(),
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
                                    }
                                });
                            });
                        ui.add_space(3.0 / 4.0 * ui.viewport_rect().height());
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

    fn save_updated_categories(&mut self) {
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

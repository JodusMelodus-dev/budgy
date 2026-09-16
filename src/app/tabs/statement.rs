use help::load_statement;
use polars::datatypes::AnyValue;

use crate::app::{Budgy, help};

impl Budgy {
    pub fn display_statement_tab(&mut self, ui: &mut egui::Ui) {
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
                                    for column_name in &column_names {
                                        row.col(|ui| {
                                            if let Ok(column) = df.column(column_name) {
                                                let value =
                                                    column.get(row_idx).unwrap_or(AnyValue::Null);

                                                ui.label(format!("{}", value));
                                            }
                                        });
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
}

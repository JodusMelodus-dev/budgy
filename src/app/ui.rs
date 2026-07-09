use polars::{datatypes::AnyValue, frame::DataFrame};

use crate::app::Budgy;

impl Budgy {
    pub fn display_dataframe(&self, ui: &mut egui::Ui, dataframe: &DataFrame) {
        let height = dataframe.height();
        let column_names = dataframe.get_column_names();

        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .columns(
                    egui_extras::Column::auto(),
                    column_names.len(),
                )
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
                                if let Ok(column) = dataframe.column(column_name) {
                                    let value = column.get(row_idx).unwrap_or(AnyValue::Null);
                                    ui.label(format!("{}", value));
                                }
                            });
                        }
                    });
                });
        });
    }
}

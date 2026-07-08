use egui::Color32;
use polars::{datatypes::AnyValue, frame::DataFrame};

pub struct Budgy {
    budget_summary: DataFrame,
}

impl Budgy {
    pub fn new(budget_summary: DataFrame) -> Self {
        Self { budget_summary }
    }
}

const COLORS: [Color32; 8] = [
    Color32::RED,
    Color32::GREEN,
    Color32::BLUE,
    Color32::PURPLE,
    Color32::YELLOW,
    Color32::MAGENTA,
    Color32::ORANGE,
    Color32::GOLD,
];

impl eframe::App for Budgy {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Budget Summary");
        ui.separator();

        let df = &self.budget_summary;
        let height = df.height();
        let column_names = df.get_column_names();

        egui::ScrollArea::horizontal().show(ui, |ui| {
            egui_extras::TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .columns(
                    egui_extras::Column::auto().at_least(100.0),
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

                        for (i, column_name) in column_names.iter().enumerate() {
                            row.col(|ui| {
                                if let Ok(column) = df.column(column_name) {
                                    let value = column.get(row_idx).unwrap_or(AnyValue::Null);
                                    ui.colored_label(COLORS[i % 8], format!("{}", value));
                                }
                            });
                        }
                    });
                });
        });
    }
}

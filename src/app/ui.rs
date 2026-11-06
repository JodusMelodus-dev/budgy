use polars::{datatypes::AnyValue, frame::DataFrame};

use crate::app::{Budgy, tabs::Tabs};

impl Budgy {
    pub fn display_dataframe(ui: &mut egui::Ui, dataframe: &DataFrame) {
        let height = dataframe.height();
        let column_names = dataframe.get_column_names();

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
                                if let Ok(column) = dataframe.column(column_name) {
                                    let value = column.get(row_idx).unwrap_or(AnyValue::Null);
                                    ui.label(format!("{value}"));
                                }
                            });
                        }
                    });
                });
        });
    }

    pub fn display_navigation_menu(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.take_available_width();
            ui.set_min_width(180.0);
            ui.spacing_mut().button_padding = egui::vec2(12.0, 3.0);

            let size = egui::vec2(ui.available_width(), 30.0);

            if ui
                .add_sized(
                    size,
                    egui::Button::image_and_text(
                        egui::include_image!("../../assets/icons/home.svg"),
                        "Dashboard",
                    )
                    .selected(self.current_tab == Tabs::Dashboard),
                )
                .clicked()
            {
                self.current_tab = Tabs::Dashboard;
            }

            if ui
                .add_sized(
                    size,
                    egui::Button::image_and_text(
                        egui::include_image!("../../assets/icons/document.svg"),
                        "Statement",
                    )
                    .selected(self.current_tab == Tabs::Statement),
                )
                .clicked()
            {
                self.current_tab = Tabs::Statement;
            }

            if ui
                .add_sized(
                    size,
                    egui::Button::image_and_text(
                        egui::include_image!("../../assets/icons/wallet.svg"),
                        "Budget",
                    )
                    .selected(self.current_tab == Tabs::Budget),
                )
                .clicked()
            {
                self.current_tab = Tabs::Budget;
            }

            if ui
                .add_sized(
                    size,
                    egui::Button::image_and_text(
                        egui::include_image!("../../assets/icons/invoice.svg"),
                        "Summary",
                    )
                    .selected(self.current_tab == Tabs::BudgetSummary),
                )
                .clicked()
            {
                self.current_tab = Tabs::BudgetSummary;
            }

            if ui
                .add_sized(
                    size,
                    egui::Button::image_and_text(
                        egui::include_image!("../../assets/icons/document.svg"),
                        "Settings",
                    )
                    .selected(self.current_tab == Tabs::Settings),
                )
                .clicked()
            {
                self.current_tab = Tabs::Settings;
            }
        });
    }
}

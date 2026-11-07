use egui::{Align, Layout, vec2};
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
            ui.spacing_mut().button_padding = egui::vec2(12.0, 3.0);

            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if ui
                    .add_sized(
                        vec2(70.0 - ui.spacing().button_padding.x * 2.0, 30.0),
                        egui::Button::new(if self.nav_collapsed {
                            "▶️"
                        } else {
                            "◀️"
                        }),
                    )
                    .on_hover_text(if self.nav_collapsed {
                        "Expand sidebar"
                    } else {
                        "Collapse sidebar"
                    })
                    .clicked()
                {
                    self.nav_collapsed = !self.nav_collapsed;
                }
            });

            let size = egui::vec2(ui.available_width(), 30.0);
            if ui
                .add_sized(
                    size,
                    egui::Button::new(if self.nav_collapsed {
                        "🏠"
                    } else {
                        "Dashboard"
                    })
                    .selected(self.current_tab == Tabs::Dashboard),
                )
                .on_hover_text("Dashboard")
                .clicked()
            {
                self.current_tab = Tabs::Dashboard;
            }

            if ui
                .add_sized(
                    size,
                    egui::Button::new(if self.nav_collapsed {
                        "📄"
                    } else {
                        "Statement"
                    })
                    .selected(self.current_tab == Tabs::Statement),
                )
                .on_hover_text("Statement")
                .clicked()
            {
                self.current_tab = Tabs::Statement;
            }

            if ui
                .add_sized(
                    size,
                    egui::Button::new(if self.nav_collapsed { "💳" } else { "Budget" })
                        .selected(self.current_tab == Tabs::Budget),
                )
                .on_hover_text("Budget")
                .clicked()
            {
                self.current_tab = Tabs::Budget;
            }

            if ui
                .add_sized(
                    size,
                    egui::Button::new(if self.nav_collapsed {
                        "📨"
                    } else {
                        "Summary"
                    })
                    .selected(self.current_tab == Tabs::BudgetSummary),
                )
                .on_hover_text("Summary")
                .clicked()
            {
                self.current_tab = Tabs::BudgetSummary;
            }

            if ui
                .add_sized(
                    size,
                    egui::Button::new(if self.nav_collapsed {
                        "⚙️"
                    } else {
                        "Settings"
                    })
                    .selected(self.current_tab == Tabs::Settings),
                )
                .on_hover_text("Settings")
                .clicked()
            {
                self.current_tab = Tabs::Settings;
            }
        });
    }
}

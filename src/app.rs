mod config;
mod help;
mod menu;
mod tabs;
mod ui;

use std::path::PathBuf;

use eframe::APP_KEY;
use egui::{Color32, Frame, ViewportCommand};
use polars::{frame::DataFrame, lazy::frame::LazyFrame};

use crate::app::{config::Config, help::load_budget, tabs::Tabs};

pub struct Budgy {
    budget_summary: Option<DataFrame>,

    budget_df: Option<DataFrame>,
    budget_lf: Option<LazyFrame>,

    statement_df: Option<DataFrame>,
    statement_lf: Option<LazyFrame>,

    previous_summary_lf: Option<LazyFrame>,

    budget_deltas: Vec<(String, f64)>,
    scroll_budget_to_bottom: bool,

    current_tab: Tabs,
    config: Config,
}

impl Budgy {
    pub fn new(mut config: Config, statement_path: Option<String>) -> Self {
        config.statement_path =
            statement_path.map_or(config.statement_path, |p| Some(PathBuf::from(p)));

        Self {
            budget_summary: None,

            budget_df: None,
            budget_lf: load_budget(),

            statement_df: None,
            statement_lf: None,

            previous_summary_lf: None,

            budget_deltas: Vec::new(),
            scroll_budget_to_bottom: false,

            current_tab: Tabs::Statement,
            config,
        }
    }
}

impl eframe::App for Budgy {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("Menu")
            .frame(Frame::default().inner_margin(5.0))
            .show_separator_line(false)
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Import statement CSV").clicked() {
                            self.import_csv();
                        }

                        if ui.button("Export summary CSV").clicked() {
                            self.export_csv();
                        }

                        ui.menu_button("Open Recent", |ui| {
                            self.open_recent(ui);
                        });

                        ui.separator();

                        if ui.button("Exit").clicked() {
                            ui.send_viewport_cmd(ViewportCommand::Close);
                        }
                    });
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.current_tab, Tabs::Statement, "Statement");
                    ui.selectable_value(&mut self.current_tab, Tabs::Budget, "Budget");
                    ui.selectable_value(&mut self.current_tab, Tabs::BudgetSummary, "Summary");
                });
            });

        egui::CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(Color32::from_rgb(32, 32, 32))
                    .corner_radius(5.0)
                    .outer_margin(5.0)
                    .inner_margin(10.0),
            )
            .show(ui, |ui| {
                match self.current_tab {
                    Tabs::Statement => self.display_statement_tab(ui),
                    Tabs::BudgetSummary => self.display_budget_summary_tab(ui),
                    Tabs::Budget => self.display_budget_tab(ui),
                };
            });
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, &self.config);
    }
}

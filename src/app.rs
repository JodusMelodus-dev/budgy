mod config;
mod help;
mod menu;
mod tabs;
mod ui;

use std::path::PathBuf;

use eframe::APP_KEY;
use egui::{Color32, Frame, Key, KeyboardShortcut, Modifiers, ViewportCommand};
use polars::{frame::DataFrame, lazy::frame::LazyFrame};

use crate::{
    app::{config::Config, help::load_budget, tabs::Tabs},
    extra_ui::ExtraUi,
};

const CTRL_O: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::O);
const CTRL_E: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::E);
const ALT_F4: KeyboardShortcut = KeyboardShortcut::new(Modifiers::ALT, Key::F4);

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
        if ui.ctx().input_mut(|i| i.consume_shortcut(&CTRL_O)) {
            self.import_csv();
        }
        if ui.ctx().input_mut(|i| i.consume_shortcut(&CTRL_E)) {
            self.export_csv();
        }

        egui::Panel::top("Menu")
            .frame(Frame::default().inner_margin(5.0))
            .show_separator_line(false)
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        ui.set_width(180.0);

                        if ui.button_with_shortcut("Import Statement", CTRL_O) {
                            self.import_csv();
                        }

                        if ui.button_with_shortcut("Export Summary", CTRL_E) {
                            self.export_csv();
                        }

                        ui.menu_button("Open Recent", |ui| {
                            ui.set_min_width(180.0);
                            self.open_recent(ui);
                        });

                        ui.separator();

                        if ui.button_with_shortcut("Exit", ALT_F4) {
                            ui.send_viewport_cmd(ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("Settings", |ui| {
                        ui.set_width(180.0);

                        if ui.button("Clear recents").clicked() {
                            self.config.recent.clear();
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

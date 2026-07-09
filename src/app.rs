mod config;
mod help;
mod tabs;
mod ui;

use std::path::PathBuf;

use eframe::APP_KEY;
use egui::ViewportCommand;
use polars::frame::DataFrame;
use rfd::FileDialog;

use crate::app::{config::Config, tabs::Tabs};

pub struct Budgy {
    budget_summary: Option<DataFrame>,
    budget: Option<DataFrame>,
    statement: Option<DataFrame>,
    lookup: Option<DataFrame>,

    current_tab: Tabs,
    config: Config,
}

impl Budgy {
    pub fn new(mut config: Config, statement_path: Option<String>) -> Self {
        config.statement_path = statement_path.map(|p| PathBuf::from(p));

        Self {
            budget_summary: None,
            budget: None,
            statement: None,
            lookup: None,

            current_tab: Tabs::Statement,
            config,
        }
    }
}

impl eframe::App for Budgy {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open File...").clicked() {
                    self.config.statement_path = FileDialog::new()
                        .add_filter("CSV Files", &["csv"])
                        .pick_file();
                }

                ui.separator();

                if ui.button("Exit").clicked() {
                    ui.send_viewport_cmd(ViewportCommand::Close);
                }
            });
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.current_tab, Tabs::Statement, "Statement");
            ui.selectable_value(&mut self.current_tab, Tabs::BudgetSummary, "Summary");
            ui.selectable_value(&mut self.current_tab, Tabs::Budget, "Budget");
        });
        ui.separator();

        match self.current_tab {
            Tabs::Statement => self.display_statements_tab(ui),
            Tabs::BudgetSummary => self.display_budget_summary_tab(ui),
            Tabs::Budget => self.display_budget_tab(ui),
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, &self.config);
    }
}

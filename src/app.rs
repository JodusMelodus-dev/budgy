mod config;
mod help;
mod tabs;
mod ui;

use std::{collections::HashMap, fs::File, path::PathBuf};

use chrono::NaiveDate;
use eframe::APP_KEY;
use egui::{Color32, Frame, ViewportCommand, epaint::Hsva};
use polars::{
    frame::DataFrame,
    io::{SerWriter, csv::write::CsvWriter},
    lazy::frame::LazyFrame,
};
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageLevel};

use crate::app::{config::Config, help::load_budget, tabs::Tabs};

pub struct Budgy {
    budget_summary: Option<DataFrame>,

    budget_df: Option<DataFrame>,
    budget_lf: Option<LazyFrame>,

    statement_df: Option<DataFrame>,
    statement_lf: Option<LazyFrame>,

    statement_deltas: HashMap<i64, String>,
    budget_deltas: Vec<(i64, String, NaiveDate, NaiveDate, f64, Hsva)>,

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

            statement_deltas: HashMap::new(),
            budget_deltas: Vec::new(),

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
                        if ui.button("Import CSV").clicked() {
                            self.config.statement_path = FileDialog::new()
                                .add_filter("CSV Files", &["csv"])
                                .pick_file();

                            if let Some(path) = &self.config.statement_path {
                                if !self.config.recents.contains(path) {
                                    self.config.recents.insert(0, path.to_path_buf());
                                }
                            }
                        }

                        if ui.button("Export CSV").clicked() {
                            let path = FileDialog::new()
                                .set_file_name("budget summary.csv")
                                .add_filter("CSV Files", &["csv"])
                                .save_file()
                                .expect("Failed to get save path");

                            let file = File::create(&path).expect("Failed to create file");
                            if let Some(mut budget_summary_df) = self.budget_summary.clone() {
                                CsvWriter::new(file)
                                    .include_header(true)
                                    .with_separator(b',')
                                    .finish(&mut budget_summary_df)
                                    .expect("Failed to export file");

                                MessageDialog::new()
                                    .set_level(MessageLevel::Info)
                                    .set_description(format!(
                                        "Successfully exported budget summary to: {}",
                                        path.to_str().unwrap()
                                    ))
                                    .set_buttons(MessageButtons::Ok)
                                    .show();
                            }
                        }

                        ui.menu_button("Open Recent", |ui| {
                            ui.vertical(|ui| {
                                for path in &self.config.recents {
                                    if ui
                                        .button(
                                            path.file_name().unwrap().to_str().unwrap_or("NULL"),
                                        )
                                        .clicked()
                                    {
                                        self.config.statement_path = Some(path.to_path_buf());
                                    }
                                }
                            });
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
                    ui.selectable_value(&mut self.current_tab, Tabs::BudgetSummary, "Summary");
                    ui.selectable_value(&mut self.current_tab, Tabs::Budget, "Budget");
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

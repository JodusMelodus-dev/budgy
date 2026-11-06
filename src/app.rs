mod file;
mod menu;
mod tabs;
mod ui;

use std::sync::mpsc::{Receiver, Sender, channel};

use egui::{Color32, Frame, Key, KeyboardShortcut, Modifiers, ViewportCommand};
use polars::{frame::DataFrame, lazy::frame::LazyFrame};

use crate::{
    app::{
        file::{BudgyFileType, ImportFile, import_budget, import_statement, import_summary},
        tabs::Tabs,
    },
    extra_ui::ExtraUi as _,
};

const CTRL_E: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::E);
const ALT_F4: KeyboardShortcut = KeyboardShortcut::new(Modifiers::ALT, Key::F4);

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Budget {
    categories: Vec<String>,
    amounts: Vec<f64>,
}

pub struct Budgy {
    budget_summary: Option<DataFrame>,

    budget_df: Option<DataFrame>,
    budget_lf: Option<LazyFrame>,

    statement_df: Option<DataFrame>,
    statement_lf: Option<LazyFrame>,

    previous_summary_lf: Option<LazyFrame>,

    current_tab: Tabs,

    // File picking
    tx: Sender<ImportFile>,
    rx: Receiver<ImportFile>,

    budget: Budget,
    last_error: Option<String>,
}

impl Budgy {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = channel();
        let budget = if let Some(storage) = cc.storage {
            eframe::get_value::<Budget>(storage, "budget").unwrap_or_default()
        } else {
            Budget::default()
        };

        Self {
            budget_summary: None,

            budget_df: None,
            budget_lf: None,

            statement_df: None,
            statement_lf: None,

            previous_summary_lf: None,

            current_tab: Tabs::Dashboard,

            tx,
            rx,

            budget,
            last_error: None,
        }
    }
}

impl eframe::App for Budgy {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if let Some(budget_df) = &self.budget_df {
            let category = budget_df
                .column("Category")
                .expect("Failed to get Category column")
                .str()
                .expect("Failed to convert to string")
                .into_iter()
                .map(|v| v.unwrap_or_default().to_owned())
                .collect::<Vec<String>>();
            let budget_amount = budget_df
                .column("Budget Amount")
                .expect("Failed to get Category column")
                .f64()
                .expect("Failed to convert to float")
                .into_iter()
                .map(|v| v.unwrap_or_default())
                .collect::<Vec<f64>>();

            self.budget.categories = category;
            self.budget.amounts = budget_amount;

            eframe::set_value(storage, "budget", &self.budget);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.ctx().input_mut(|i| i.consume_shortcut(&CTRL_E)) {
            self.export_csv();
        }

        while let Ok(file) = self.rx.try_recv() {
            match file.file_type {
                BudgyFileType::Budget => {
                    self.budget_lf = import_budget(file.bytes)
                        .map_err(|e| self.last_error = Some(e.to_string()))
                        .ok();
                }
                BudgyFileType::Statement => {
                    self.statement_lf = import_statement(file.bytes)
                        .map_err(|e| self.last_error = Some(e.to_string()))
                        .ok();
                }
                BudgyFileType::Summary => {
                    self.previous_summary_lf = import_summary(file.bytes)
                        .map_err(|e| self.last_error = Some(e.to_string()))
                        .ok();
                }
            }
        }

        egui::Panel::top("Menu")
            .frame(Frame::default().inner_margin(5.0))
            .show_separator_line(false)
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        ui.set_width(180.0);

                        if ui.button("Load previous summary").clicked() {
                            self.open_csv_file(BudgyFileType::Summary);
                        }

                        if ui.button_with_shortcut("Export Summary", CTRL_E) {
                            self.export_csv();
                        }

                        ui.separator();

                        if ui.button_with_shortcut("Exit", ALT_F4) {
                            ui.send_viewport_cmd(ViewportCommand::Close);
                        }
                    });
                    ui.menu_button("Settings", |ui| {
                        ui.set_width(180.0);
                    });
                    ui.menu_button("Help", |ui| {
                        ui.set_width(180.0);

                        if ui.button("Open Website").clicked() {
                            Self::open_help();
                        }
                    });
                });
            });

        egui::Panel::left("navigation")
            .frame(
                Frame::default()
                    .fill(Color32::from_rgb(32, 32, 32))
                    .corner_radius(5.0)
                    .outer_margin(5.0)
                    .inner_margin(10.0),
            )
            .show(ui, |ui| {
                self.display_navigation_menu(ui);
            });

        egui::CentralPanel::default()
            .frame(
                Frame::default()
                    .fill(Color32::from_rgb(32, 32, 32))
                    .corner_radius(5.0)
                    .outer_margin(5.0)
                    .inner_margin(10.0),
            )
            .show(ui, |ui| match self.current_tab {
                Tabs::Statement => self.display_statement_tab(ui),
                Tabs::BudgetSummary => self.display_budget_summary_tab(ui),
                Tabs::Budget => self.display_budget_tab(ui).unwrap(),
                Tabs::Dashboard => self.display_dashboard(ui),
                Tabs::Settings => self.display_settings(ui),
            });
    }
}

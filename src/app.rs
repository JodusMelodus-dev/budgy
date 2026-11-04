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
    rows: Vec<(String, f64)>,
}

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

    // File picking
    tx: Sender<ImportFile>,
    rx: Receiver<ImportFile>,

    budget: Budget,
    last_error: Option<String>,
}

impl Budgy {
    pub fn new() -> Self {
        let (tx, rx) = channel();

        Self {
            budget_summary: None,

            budget_df: None,
            budget_lf: None,

            statement_df: None,
            statement_lf: None,

            previous_summary_lf: None,

            budget_deltas: Vec::new(),
            scroll_budget_to_bottom: false,

            current_tab: Tabs::Statement,

            tx,
            rx,

            budget: Budget::default(),
            last_error: None,
        }
    }
}

impl eframe::App for Budgy {
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
                Tabs::Budget => self.display_budget_tab(ui),
            });
    }
}

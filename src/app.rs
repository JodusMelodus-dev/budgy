mod config;
mod help;

use std::path::{Path, PathBuf};

use eframe::APP_KEY;
use egui::ViewportCommand;
use polars::{
    chunked_array::ops::SortMultipleOptions,
    datatypes::AnyValue,
    frame::{DataFrame, UniqueKeepStrategy},
    lazy::{
        dsl::{col, lit},
        frame::IntoLazy,
    },
    prelude::JoinType,
};
use rfd::FileDialog;

use help::{generate_undefined_categories, load_budget, load_lookup, load_statement};

use crate::app::config::Config;

pub struct Budgy {
    budget_summary: Option<DataFrame>,
    config: Config,
}

impl Budgy {
    pub fn new(mut config: Config, statement_path: Option<String>) -> Self {
        config.statement_path = statement_path.map(|p| PathBuf::from(p));

        Self {
            budget_summary: None,
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

        if self.budget_summary.is_none() {
            if let Some(path) = &self.config.statement_path {
                let lookup = load_lookup(Path::new("lookup.csv"))
                    .expect("Failed to load lookup")
                    .with_column(lit(1).alias("Join Key"));

                let mut statement = load_statement(&path)
                    .expect("Failed to load statement")
                    .with_column(lit(1).alias("Join Key"));

                let combined = statement.join(
                    lookup,
                    [col("Join Key")],
                    [col("Join Key")],
                    JoinType::Inner.into(),
                );
                let matched = combined.filter(col("Description").str().contains(col("Mask"), true));
                statement = matched
                    .drop([col("Join Key")])
                    .unique(Some(vec!["Nr".to_string()]), UniqueKeepStrategy::First);

                let budget = load_budget(Path::new("budget.csv")).expect("Failed to load budget");

                let summary = statement
                    .clone()
                    .lazy()
                    .filter(col("New Category").is_not_null())
                    .left_join(budget, col("New Category"), col("Category"))
                    .filter(
                        col("Date")
                            .gt_eq(col("Start Date"))
                            .and(col("Date").lt_eq("End Date")),
                    )
                    .collect()
                    .expect("Failed to collect summray");

                let result = summary
                    .clone()
                    .lazy()
                    .group_by([col("New Category"), col("Start Date"), col("End Date")])
                    .agg([
                        col("Money In").fill_null(lit(0.0)).sum().alias("Total In"),
                        col("Money Out")
                            .fill_null(lit(0.0))
                            .sum()
                            .alias("Total Out"),
                        col("Fee").fill_null(lit(0.0)).sum().alias("Total Fees"),
                        (col("Money In").fill_null(lit(0.0)).sum()
                            + col("Money Out").fill_null(lit(0.0)).sum()
                            + col("Fee").fill_null(lit(0.0)).sum())
                        .alias("Net Total"),
                        (col("Budget Amount") * col("Month Difference"))
                            .max()
                            .alias("Budgetted Amount"),
                        ((col("Money In").fill_null(lit(0.0)).sum()
                            + col("Money Out").fill_null(lit(0.0)).sum()
                            + col("Fee").fill_null(lit(0.0)).sum())
                            - (col("Budget Amount") * col("Month Difference")).max())
                        .alias("NET BUDGET"),
                    ])
                    .sort(
                        ["Start Date", "New Category"],
                        SortMultipleOptions::new().with_order_descending(false),
                    );

                let final_result = result
                    .group_by(["New Category"])
                    .agg([(col("NET BUDGET").sum()).alias("Budget Balance")])
                    .sort(
                        ["Budget Balance"],
                        SortMultipleOptions::new().with_order_descending(false),
                    );

                generate_undefined_categories(statement).expect("Failed to get undef categories");

                self.budget_summary = Some(
                    final_result
                        .collect()
                        .expect("Failed to generate budget summary"),
                );
            }
        }

        if let Some(df) = &self.budget_summary {
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

                            for column_name in &column_names {
                                row.col(|ui| {
                                    if let Ok(column) = df.column(column_name) {
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

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, &self.config);
    }
}

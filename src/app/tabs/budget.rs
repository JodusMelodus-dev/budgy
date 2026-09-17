use std::{fs::File, path::PathBuf};

use egui::Align;
use polars::{
    frame::{DataFrame, column::Column},
    io::{SerWriter, csv::write::CsvWriter},
};

use crate::app::{Budgy, help::load_budget};

impl Budgy {
    pub fn display_budget_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Add Row").clicked() {
                self.budget_deltas.push((
                    "".to_string(),
                    0.0,
                ));
                self.scroll_budget_to_bottom = true;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                if ui.button("Save").clicked() {
                    self.save_updated_budget();
                }
            });
        });

        ui.separator();

        if let Some(df) = &self.budget_df {
            let column_names = df.get_column_names();

            egui::ScrollArea::horizontal()
                .id_salt("budget")
                .show(ui, |ui| {
                    let mut table = egui_extras::TableBuilder::new(ui).striped(true);

                    if self.scroll_budget_to_bottom {
                        table = table.scroll_to_row(self.budget_deltas.len(), Some(Align::BOTTOM));
                        self.scroll_budget_to_bottom = false;
                    }

                    table
                        .resizable(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .columns(egui_extras::Column::auto(), column_names.len())
                        .header(20.0, |mut header| {
                            for name in &column_names[..column_names.len()] {
                                header.col(|ui| {
                                    ui.strong(name.to_string());
                                });
                            }
                        })
                        .body(|body| {
                            if self.budget_deltas.len() > 0 {
                                body.rows(22.0, self.budget_deltas.len(), |mut row| {
                                    let row_idx = row.index();

                                    row.col(|ui| {
                                        let mut text = self.budget_deltas[row_idx].0.clone();
                                        ui.text_edit_singleline(&mut text);
                                        self.budget_deltas[row_idx].0 = text;
                                    });
                                    row.col(|ui| {
                                        let mut text = self.budget_deltas[row_idx].1.to_string();
                                        ui.text_edit_singleline(&mut text);
                                        self.budget_deltas[row_idx].1 =
                                            text.parse::<f64>().unwrap_or(0.0);
                                    });
                                });
                            }
                        });
                });
        } else if let Some(lf) = self.budget_lf.clone() {
            self.budget_df = Some(lf.collect().expect("Failed to collect budget lazy frame"));

            if let Some(df) = &self.budget_df {
                let categories = df.column("Category").unwrap().str().unwrap();
                let budget_amounts = df.column("Budget Amount").unwrap().f64().unwrap();

                for i in 0..df.height() {
                    let category = categories.get(i).unwrap_or("").to_string();
                    let budget_amount = budget_amounts.get(i).unwrap_or(0.0);

                    self.budget_deltas
                        .push((category, budget_amount));
                }
            }
        } else {
            self.budget_lf = load_budget();
        }
    }

    fn save_updated_budget(&mut self) {
        let mut categories = Vec::with_capacity(self.budget_deltas.len());
        let mut budget_amounts = Vec::with_capacity(self.budget_deltas.len());

        for (category, budget_amount) in self.budget_deltas.drain(..) {
            categories.push(category);
            budget_amounts.push(budget_amount);
        }

        let mut table = DataFrame::new(vec![
            Column::new("Category".into(), categories),
            Column::new("Budget Amount".into(), budget_amounts),
        ])
        .unwrap();

        let path = PathBuf::from("budget.csv");
        let file = File::create(path).unwrap();
        CsvWriter::new(file)
            .include_header(true)
            .with_separator(b',')
            .finish(&mut table)
            .unwrap();
    }
}

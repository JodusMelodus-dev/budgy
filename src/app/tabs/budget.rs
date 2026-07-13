use std::{fs::File, path::PathBuf, str::FromStr};

use chrono::NaiveDate;
use egui::{Color32, epaint::Hsva};
use polars::{
    frame::{DataFrame, column::Column},
    io::{SerWriter, csv::write::CsvWriter},
};

use crate::app::{Budgy, help::load_budget};

impl Budgy {
    pub fn display_budget_tab(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
            if ui.button("Save").clicked() {
                self.save_updated_budget();
            }
        });

        ui.separator();

        if let Some(df) = &self.budget_df {
            let column_names = df.get_column_names();

            egui::ScrollArea::horizontal().show(ui, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .columns(egui_extras::Column::auto(), column_names.len() - 1)
                    .header(20.0, |mut header| {
                        for name in &column_names[..column_names.len() - 1] {
                            header.col(|ui| {
                                ui.strong(name.to_string());
                            });
                        }
                    })
                    .body(|body| {
                        if self.budget_deltas.len() > 0 {
                            body.rows(22.0, self.budget_deltas.len() - 1, |mut row| {
                                let row_idx = row.index();

                                row.col(|ui| {
                                    ui.label(format!("{}", self.budget_deltas[row_idx].0));
                                });
                                row.col(|ui| {
                                    let mut text = self.budget_deltas[row_idx].1.clone();
                                    ui.text_edit_singleline(&mut text);
                                    self.budget_deltas[row_idx].1 = text;
                                });
                                row.col(|ui| {
                                    let mut text = self.budget_deltas[row_idx].2.to_string();
                                    ui.text_edit_singleline(&mut text);
                                    self.budget_deltas[row_idx].2 =
                                        NaiveDate::from_str(&text).unwrap();
                                });
                                row.col(|ui| {
                                    let mut text = self.budget_deltas[row_idx].3.to_string();
                                    ui.text_edit_singleline(&mut text);
                                    self.budget_deltas[row_idx].3 =
                                        NaiveDate::from_str(&text).unwrap();
                                });
                                row.col(|ui| {
                                    let mut text = self.budget_deltas[row_idx].4.to_string();
                                    ui.text_edit_singleline(&mut text);
                                    self.budget_deltas[row_idx].4 =
                                        text.parse::<f64>().unwrap_or(0.0);
                                });
                                row.col(|ui| {
                                    let color = Color32::from(self.budget_deltas[row_idx].5);
                                    ui.color_edit_button_hsva(&mut self.budget_deltas[row_idx].5)
                                        .labelled_by(ui.label(color.to_hex()).id);
                                });
                            });
                        }
                    });
            });
        } else if let Some(lf) = self.budget_lf.clone() {
            self.budget_df = Some(lf.collect().expect("Failed to collect budget lazy frame"));

            if let Some(df) = &self.budget_df {
                let nrs = df.column("B_Nr").unwrap().i64().unwrap();
                let categories = df.column("Category").unwrap().str().unwrap();
                let start_dates = df.column("Start Date").unwrap().date().unwrap();
                let end_dates = df.column("End Date").unwrap().date().unwrap();
                let budget_amounts = df.column("Budget Amount").unwrap().f64().unwrap();
                let colors = df.column("Color").unwrap().str().unwrap();

                for i in 0..df.height() {
                    let nr = nrs.get(i).unwrap_or(0);
                    let category = categories.get(i).unwrap_or("").to_string();
                    let start_date =
                        NaiveDate::from_epoch_days(start_dates.get(i).unwrap_or(0)).unwrap();
                    let end_date =
                        NaiveDate::from_epoch_days(end_dates.get(i).unwrap_or(0)).unwrap();
                    let budget_amount = budget_amounts.get(i).unwrap_or(0.0);
                    let color = Hsva::from(
                        Color32::from_hex(colors.get(i).unwrap_or("#FFFFFF"))
                            .unwrap_or(Color32::WHITE),
                    );

                    self.budget_deltas.push((
                        nr,
                        category,
                        start_date,
                        end_date,
                        budget_amount,
                        color,
                    ));
                }
            }
        } else {
            self.budget_lf = load_budget();
        }
    }

    fn save_updated_budget(&mut self) {
        let mut nrs = Vec::with_capacity(self.budget_deltas.len());
        let mut categories = Vec::with_capacity(self.budget_deltas.len());
        let mut start_dates = Vec::with_capacity(self.budget_deltas.len());
        let mut end_dates = Vec::with_capacity(self.budget_deltas.len());
        let mut budget_amounts = Vec::with_capacity(self.budget_deltas.len());
        let mut colors = Vec::with_capacity(self.budget_deltas.len());

        for (nr, category, start_date, end_date, budget_amount, color) in
            self.budget_deltas.drain(..)
        {
            nrs.push(nr);
            categories.push(category);
            start_dates.push(start_date);
            end_dates.push(end_date);
            budget_amounts.push(budget_amount);
            colors.push(Color32::from(color).to_hex());
        }

        let mut table = DataFrame::new(vec![
            Column::new("B_Nr".into(), nrs),
            Column::new("Category".into(), categories),
            Column::new("Start Date".into(), start_dates),
            Column::new("End Date".into(), end_dates),
            Column::new("Budget Amount".into(), budget_amounts),
            Column::new("Color".into(), colors),
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

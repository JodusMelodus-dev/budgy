use help::load_statement;

use crate::app::{Budgy, help};

impl Budgy {
    pub fn display_statement_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(df) = &self.statement_df {
            self.display_dataframe(ui, df);
        } else if let Some(lf) = self.statement_lf.clone() {
            self.statement_df = Some(
                lf.collect()
                    .expect("Failed to collect statement lazy frame"),
            );
        } else if let Some(path) = &self.config.statement_path {
            self.statement_lf = load_statement(path);
        } else {
            ui.heading("Open a statement to get started");
        }
    }
}

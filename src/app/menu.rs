use crate::app::{Budgy, BudgyFileType, file::export_csv};
use polars::lazy::{dsl::col, frame::IntoLazy as _};

#[cfg(target_arch = "wasm32")]
pub fn execute<F: std::future::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn execute<F: std::future::Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || pollster::block_on(f));
}

impl Budgy {
    pub fn open_csv_file(&self, file_type: BudgyFileType) {
        let tx = self.tx.clone();

        execute(async move {
            let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("CSV Files", &["csv"])
                .pick_file()
                .await
            else {
                log::error!("Failed to read CSV file");
                return;
            };

            let bytes = file.read().await;
            if let Err(e) = tx.send(super::ImportFile { file_type, bytes }) {
                log::error!("Failed to send loaded file data: {e}");
            }
        });
    }

    pub fn export_csv(&self) {
        let Some(budget_summary_df) = self.budget_summary.clone() else {
            log::error!("No budget summary available");
            return;
        };

        let Some(mut summary) = budget_summary_df
            .lazy()
            .select([col("Category"), col("Balance")])
            .collect()
            .ok()
        else {
            log::error!("Failed to serialize CSV file");
            return;
        };

        execute(async move {
            if let Some(file) = rfd::AsyncFileDialog::new()
                .set_title("Export summary")
                .set_file_name("summary.csv")
                .save_file()
                .await
            {
                export_csv(&file, &mut summary)
                    .await
                    .expect("Failed to save file");
            }
        });
    }

    pub fn open_web_app() {
        if let Err(e) = webbrowser::open("https://jodusmelodus.github.io/budgy/") {
            log::error!("Failed to open web link: {e}");
        }
    }
}

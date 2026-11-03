use crate::app::{Budgy, LoadedFileType};
use polars::lazy::{dsl::col, frame::IntoLazy as _};

use crate::app::help::save_file;

#[cfg(target_arch = "wasm32")]
pub fn execute<F: std::future::Future<Output = ()> + 'static>(f: F) {
    wasm_bindgen_futures::spawn_local(f);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn execute<F: std::future::Future<Output = ()> + Send + 'static>(f: F) {
    std::thread::spawn(move || pollster::block_on(f));
}

impl Budgy {
    pub fn open_csv_file(&self, file_type: LoadedFileType) {
        let tx = self.tx.clone();

        execute(async move {
            if let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("CSV Files", &["csv"])
                .pick_file()
                .await
            {
                let bytes = file.read().await;
                if let Err(e) = tx.send(super::LoadedFile { file_type, bytes }) {
                    log::error!("Failed to send loaded file data: {e}");
                }
            }
        });
    }

    pub fn export_csv(&self) {
        if let Some(budget_summary_df) = self.budget_summary.clone() {
            let Some(mut summary) = budget_summary_df
                .lazy()
                .select([col("Category"), col("Balance")])
                .collect()
                .ok()
            else {
                log::error!("Failed to serialize CSV file");
                return;
            };
            save_file("summary.csv".to_owned(), &mut summary);
        }
    }

    pub fn open_help() {
        if let Err(e) = webbrowser::open("https://jodusmelodus-dev.github.io/budgy/") {
            log::error!("Failed to open web link: {e}");
        }
    }
}

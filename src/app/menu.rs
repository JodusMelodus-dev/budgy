use crate::app::{Budgy, LoadedFileType};
use polars::lazy::{
        dsl::col,
        frame::IntoLazy,
    };

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
    pub fn open_csv_file(&mut self, file_type: LoadedFileType) {
        let tx = self.tx.clone();

        execute(async move {
            if let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("CSV Files", &["csv"])
                .pick_file()
                .await
            {
                let bytes = file.read().await;
                let _ = tx.send(super::LoadedFile { file_type, bytes });
            }
        });
    }

    pub fn export_csv(&mut self) {


        
        if let Some(budget_summary_df) = self.budget_summary.clone() {
            let mut summary = budget_summary_df
            .lazy()
            .select([col("Category"), col("Balance")])
            .collect()
            .unwrap();
            save_file("summary.csv".to_string(), &mut summary);
        }
    }

    pub fn open_help(&self) {
        webbrowser::open("https://jodusmelodus-dev.github.io/budgy/").unwrap();
    }
}

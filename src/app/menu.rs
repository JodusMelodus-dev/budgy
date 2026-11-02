use crate::app::{Budgy, LoadedFileType};

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
        todo!("Export not implemented for web yet");
        // let path = match self.config.statement_path.clone() {
        //     Some(mut p) => {
        //         p.set_file_name("summary.csv");
        //         p
        //     }
        //     None => FileDialog::new()
        //         .set_file_name("summary.csv")
        //         .add_filter("CSV Files", &["csv"])
        //         .save_file()
        //         .expect("No path selected"),
        // };

        // let file = File::create(&path).expect("Failed to create file");
        // if let Some(budget_summary_df) = self.budget_summary.clone() {
        //     let mut summary = budget_summary_df
        //         .lazy()
        //         .select([col("Category"), col("Balance")])
        //         .collect()
        //         .unwrap();

        //     CsvWriter::new(file)
        //         .include_header(true)
        //         .with_separator(b',')
        //         .finish(&mut summary)
        //         .expect("Failed to export file");
        // }
    }

    pub fn open_help(&self) {
        webbrowser::open("https://jodusmelodus-dev.github.io/budgy/").unwrap();
    }
}

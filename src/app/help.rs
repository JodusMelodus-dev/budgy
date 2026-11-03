use std::io::Cursor;

use polars::{
    frame::{DataFrame, column::Column},
    io::{
        SerReader as _, SerWriter as _,
        csv::{
            read::{CsvParseOptions, CsvReadOptions},
            write::CsvWriter,
        },
    },
    lazy::{
        dsl::col,
        frame::{IntoLazy as _, LazyFrame},
    },
};

use crate::app::{Budgy, menu::execute};

pub fn load_statement(bytes: Vec<u8>) -> Option<LazyFrame> {
    let Ok(data) = CsvReadOptions::default()
        .with_has_header(true)
        .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
        .into_reader_with_file_handle(Cursor::new(bytes))
        .finish()
    else {
        log::error!("Failed to read CSV statement file");
        return None;
    };

    let data = data.lazy().filter(
        col("Category")
            .is_not_null()
            .and(col("Category").is_not_null()),
    );

    Some(data)
}

pub fn load_budget(bytes: Vec<u8>) -> Option<LazyFrame> {
    if bytes.is_empty() {
        let mut blank_budget = DataFrame::new(vec![
            Column::new("Category".into(), [""]),
            Column::new("Budget Amount".into(), [0.0]),
        ])
        .expect("Failed to create blank budget");

        CsvWriter::new(Cursor::new(bytes))
            .include_header(true)
            .with_separator(b',')
            .finish(&mut blank_budget)
            .expect("Failed to save blank budget");
        None
    } else {
        let budget = CsvReadOptions::default()
            .with_has_header(true)
            .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
            .into_reader_with_file_handle(Cursor::new(bytes))
            .finish()
            .expect("Failed to load budget")
            .lazy();

        Some(budget)
    }
}

pub fn save_file(file_name: String, data_frame: &mut DataFrame) {
    let mut buf = Vec::new();
    if let Err(e) = CsvWriter::new(Cursor::new(&mut buf))
        .include_header(true)
        .with_separator(b',')
        .finish(data_frame)
    {
        log::error!("Failed to serialize CSV: {e}");
        return;
    }

    execute(async move {
        let Some(file) = rfd::AsyncFileDialog::new()
            .set_file_name(file_name)
            .add_filter("CSV", &["csv"])
            .save_file()
            .await
        else {
            log::error!("Failed to save file");
            return;
        };

        if let Err(e) = file.write(&buf).await {
            log::error!("Failed to write CSV file: {e}");
        }
    });
}

impl Budgy {
    pub fn load_previous_summary(&self, bytes: Vec<u8>) -> Option<LazyFrame> {
        if bytes.is_empty() {
            let Some(budget) = &self.budget_df else {
                log::error!("No budget lazy frame");
                return None;
            };
            let Ok(categories) = budget.column("Category") else {
                log::error!("Failed to get Category column in Budget");
                return None;
            };

            let Ok(mut blank_summary) = DataFrame::new(vec![
                categories.clone().with_name("Category".into()),
                Column::new("Balance".into(), vec![0.0; categories.len()]),
            ]) else {
                log::error!("Failed to create blank summary");
                return None;
            };

            save_file("summary.csv".to_owned(), &mut blank_summary);
            Some(blank_summary.lazy())
        } else {
            match CsvReadOptions::default()
                .with_has_header(true)
                .into_reader_with_file_handle(Cursor::new(bytes))
                .finish()
            {
                Ok(df) => Some(df.lazy()),
                Err(e) => {
                    log::error!("Failed to read previous summary: {e}");
                    None
                }
            }
        }
    }
}

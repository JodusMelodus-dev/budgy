use core::panic;
use std::io::Cursor;

use polars::{
    frame::{DataFrame, column::Column},
    io::{
        SerReader, SerWriter,
        csv::{
            read::{CsvParseOptions, CsvReadOptions},
            write::CsvWriter,
        },
    },
    lazy::{
        dsl::col,
        frame::{IntoLazy, LazyFrame},
    },
};

use crate::app::{Budgy, menu::execute};

pub fn load_statement(bytes: Vec<u8>) -> Option<LazyFrame> {
    let mut data = CsvReadOptions::default()
        .with_has_header(true)
        .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
        .into_reader_with_file_handle(Cursor::new(bytes))
        .finish()
        .expect("Failed to load statement")
        .lazy();

    data = data.filter(
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

impl Budgy {
    pub fn load_previous_summary(&self, bytes: Vec<u8>) -> Option<LazyFrame> {
        let summary = if bytes.is_empty() {
            if let Some(budget) = &self.budget_df {
                let categories = budget.column("Category").unwrap();

                let mut blank_summary = DataFrame::new(vec![
                    categories.clone().with_name("Category".into()),
                    Column::new("Balance".into(), vec![0.0; categories.len()]),
                ])
                .expect("Failed to create blank summary");

                let mut buf = Vec::new();
                CsvWriter::new(&mut buf)
                    .include_header(true)
                    .with_separator(b',')
                    .finish(&mut blank_summary)
                    .expect("Failed to save blank summary");

                execute(async move {
                    if let Some(file) = rfd::AsyncFileDialog::new()
                        .set_file_name("summary.csv")
                        .add_filter("CSV", &["csv"])
                        .save_file()
                        .await
                    {
                        file.write(&buf).await.unwrap();
                    }
                });

                blank_summary.lazy()
            } else {
                panic!("ARH");
            }
        } else {
            CsvReadOptions::default()
                .with_has_header(true)
                .into_reader_with_file_handle(Cursor::new(bytes))
                .finish()
                .expect("Failed to load previous summary")
                .lazy()
        };

        Some(summary)
    }
}

use core::panic;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

use polars::{
    datatypes::DataType,
    frame::{DataFrame, column::Column},
    io::{SerWriter, csv::write::CsvWriter},
    lazy::{
        dsl::col,
        frame::{IntoLazy, LazyCsvReader, LazyFileListReader, LazyFrame},
    },
};

pub fn load_statement(path: &Path) -> Option<LazyFrame> {
    let mut data = LazyCsvReader::new(path)
        .with_has_header(true)
        .with_try_parse_dates(true)
        .finish()
        .expect("Failed to load statement");

    data = data.filter(
        col("Category")
            .is_not_null()
            .and(col("Parent Category").is_not_null()),
    );

    Some(data)
}

pub fn load_budget() -> Option<LazyFrame> {
    let path = PathBuf::from("budget.csv");

    if !path.exists() {
        let file = File::create(&path).expect("Failed to create 'budget.csv'");

        let mut blank_budget = DataFrame::new(vec![
            Column::new_empty("Category".into(), &DataType::String),
            Column::new_empty("Budget Amount".into(), &DataType::Float32),
        ])
        .expect("Failed to create blank budget");

        CsvWriter::new(file)
            .include_header(true)
            .with_separator(b',')
            .finish(&mut blank_budget)
            .expect("Failed to save blank budget");

        println!("Populate budget.csv before running Budgy again.");
        None
    } else {
        let budget = LazyCsvReader::new(path)
            .with_has_header(true)
            .with_try_parse_dates(true)
            .finish()
            .expect("Failed to load budget");

        Some(budget)
    }
}

pub fn load_previous_summary() -> Option<LazyFrame> {
    let path = PathBuf::from("summary.csv");

    let summary = if !path.exists() {
        let file = File::create(&path).expect("Failed to create blank summary");

        if let Some(budget) = load_budget() {
            let b = budget.collect().unwrap();
            let categories = b.column("Category").unwrap();

            let mut blank_summary = DataFrame::new(vec![
                categories.clone().with_name("Parent Category".into()),
                Column::new("Money In".into(), vec![0.0; categories.len()]),
                Column::new("Money Out".into(), vec![0.0; categories.len()]),
                Column::new("Fee".into(), vec![0.0; categories.len()]),
                Column::new("Budget Amount".into(), vec![0.0; categories.len()]),
                Column::new("Balance".into(), vec![0.0; categories.len()]),
            ])
            .expect("Failed to create blank summary");

            CsvWriter::new(file)
                .include_header(true)
                .with_separator(b',')
                .finish(&mut blank_summary)
                .expect("Failed to save blank summary");

            blank_summary.lazy()
        } else {
            panic!("ARH");
        }
    } else {
        LazyCsvReader::new(path)
            .with_has_header(true)
            .finish()
            .expect("Failed to load previous summary")
    };

    Some(summary)
}

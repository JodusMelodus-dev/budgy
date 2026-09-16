use std::{
    fs::File,
    path::{Path, PathBuf},
};

use polars::{
    datatypes::DataType,
    frame::{DataFrame, column::Column},
    io::{SerWriter, csv::write::CsvWriter},
    lazy::{
        dsl::{col, lit},
        frame::{LazyCsvReader, LazyFileListReader, LazyFrame},
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
            Column::new_empty("B_Nr".into(), &DataType::Int64),
            Column::new_empty("Category".into(), &DataType::String),
            Column::new_empty("Start Date".into(), &DataType::Date),
            Column::new_empty("End Date".into(), &DataType::Date),
            Column::new_empty("Budget Amount".into(), &DataType::Float32),
            Column::new_empty("Color".into(), &DataType::String),
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
        let filtered_budget = budget.with_column(
            (col("End Date").dt().month() - col("Start Date").dt().month() + lit(1))
                .alias("Month Difference"),
        );

        Some(filtered_budget)
    }
}

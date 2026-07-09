use std::{
    fs::File,
    path::{Path, PathBuf},
};

use polars::{
    datatypes::{DataType, PlSmallStr},
    frame::{DataFrame, column::Column},
    io::{SerWriter, csv::write::CsvWriter},
    lazy::{
        dsl::{StrptimeOptions, col, dtype_col, lit},
        frame::{LazyCsvReader, LazyFileListReader, LazyFrame},
    },
};

pub fn load_lookup() -> Option<LazyFrame> {
    let path = PathBuf::from("lookup.csv");

    if !path.exists() {
        let file = File::create(&path).expect("Failed to create 'lookup.csv'");

        let mut blank_lookup = DataFrame::new(vec![
            Column::new_empty("Mask".into(), &DataType::String),
            Column::new_empty("New Category".into(), &DataType::String),
        ])
        .expect("Failed to create blank lookup");

        CsvWriter::new(file)
            .include_header(true)
            .with_separator(b',')
            .finish(&mut blank_lookup)
            .expect("Failed to save blank lookup");

        println!("Populate lookup.csv before running Budgy again.");
        None
    } else {
        let mut lookup = LazyCsvReader::new(path)
            .with_has_header(true)
            .finish()
            .expect("Failed to load lookup")
            .with_columns([dtype_col(&DataType::String).str().strip_chars(lit(""))]);
        let schema = lookup
            .collect_schema()
            .expect("Failed to get lookup schema");
        let old_names = schema
            .iter_names()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let new_names = old_names
            .iter()
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();
        Some(
            lookup
                .rename(&old_names, &new_names, false)
                .with_column(lit(1).alias("Join Key")),
        )
    }
}

pub fn load_statement(path: &Path) -> Option<LazyFrame> {
    let mut data = LazyCsvReader::new(path)
        .with_has_header(true)
        .finish()
        .expect("Failed to load statement");
    let schema = data
        .collect_schema()
        .expect("Failed to load statement schema");
    let old_names = schema
        .iter_names()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let new_names = old_names
        .iter()
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();
    let new_data = data.rename(&old_names, &new_names, false);

    let data_with_dates =
        new_data.with_columns([(col("Posting Date").str().to_date(StrptimeOptions {
            format: Some(PlSmallStr::from_str("%Y-%m-%d")),
            strict: true,
            exact: true,
            ..Default::default()
        }))
        .alias("Date")]);

    let filtered_data = data_with_dates.filter(
        col("Money In")
            .is_not_null()
            .or(col("Money Out").is_not_null().or(col("Fee").is_not_null())),
    );
    Some(filtered_data.with_column(lit(1).alias("Join Key")))
}

pub fn load_budget() -> Option<LazyFrame> {
    let path = PathBuf::from("budget.csv");

    if !path.exists() {
        let file = File::create(&path).expect("Failed to create 'budget.csv'");

        let mut blank_budget = DataFrame::new(vec![
            Column::new_empty("Category".into(), &DataType::String),
            Column::new_empty("Start".into(), &DataType::Date),
            Column::new_empty("End".into(), &DataType::Date),
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
            .finish()
            .expect("Failed to load budget");
        let budget_with_dates = budget.with_columns([
            (col("Start").str().to_date(StrptimeOptions {
                format: Some(PlSmallStr::from_str("%Y-%m-%d")),
                strict: true,
                exact: true,
                ..Default::default()
            }))
            .alias("Start Date"),
            (col("End").str().to_date(StrptimeOptions {
                format: Some(PlSmallStr::from_str("%Y-%m-%d")),
                strict: true,
                exact: true,
                ..Default::default()
            }))
            .alias("End Date"),
        ]);
        let filtered_budget = budget_with_dates
            .with_column(
                (col("End Date").dt().month() - col("Start Date").dt().month() + lit(1))
                    .alias("Month Difference"),
            )
            .select([
                col("Category"),
                col("Budget Amount"),
                col("Start Date"),
                col("End Date"),
                col("Month Difference"),
            ]);

        Some(filtered_budget)
    }
}

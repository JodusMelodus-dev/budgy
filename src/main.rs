use std::{
    env::{self, args, set_var},
    fs::File,
    io::{Write, stdin, stdout},
    path::Path,
};

use polars::{
    chunked_array::ops::SortMultipleOptions,
    datatypes::{DataType, PlSmallStr},
    error::{PolarsError, PolarsResult},
    frame::{DataFrame, UniqueKeepStrategy, column::Column},
    io::{SerWriter, csv::write::CsvWriter},
    lazy::{
        dsl::{StrptimeOptions, col, dtype_col, lit},
        frame::{IntoLazy, LazyCsvReader, LazyFileListReader, LazyFrame},
    },
    prelude::JoinType,
};

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    stdout().flush().expect("Failed to flush");
    let mut buffer = String::new();
    stdin().read_line(&mut buffer).expect("Failed to read line");
    buffer.trim().to_string()
}

fn load_lookup(path: &Path) -> PolarsResult<LazyFrame> {
    if !path.exists() {
        let file = File::create(path).expect("Failed to create 'lookup.csv'");

        let mut blank_lookup = DataFrame::new(vec![
            Column::new_empty("Mask".into(), &DataType::String),
            Column::new_empty("New Category".into(), &DataType::String),
        ])?;

        CsvWriter::new(file)
            .include_header(true)
            .with_separator(b',')
            .finish(&mut blank_lookup)?;

        println!("Populate lookup.csv before running Budgy again.");
        Err(PolarsError::ComputeError("Empty lookup.csv".into()))?
    }

    let mut lookup = LazyCsvReader::new(path)
        .with_has_header(true)
        .finish()?
        .with_columns([dtype_col(&DataType::String).str().strip_chars(lit(""))]);
    let schema = lookup.collect_schema()?;
    let old_names = schema
        .iter_names()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let new_names = old_names
        .iter()
        .map(|s| s.trim().to_string())
        .collect::<Vec<String>>();
    Ok(lookup.rename(&old_names, &new_names, false))
}

fn load_statement(path: &Path) -> PolarsResult<LazyFrame> {
    let mut data = LazyCsvReader::new(path).with_has_header(true).finish()?;
    let schema = data.collect_schema()?;
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
    Ok(filtered_data)
}

fn load_budget(path: &Path) -> PolarsResult<LazyFrame> {
    if !path.exists() {
        let file = File::create(path).expect("Failed to create 'budget.csv'");

        let mut blank_budget = DataFrame::new(vec![
            Column::new_empty("Category".into(), &DataType::String),
            Column::new_empty("Start".into(), &DataType::Date),
            Column::new_empty("End".into(), &DataType::Date),
            Column::new_empty("Budget Amount".into(), &DataType::Float32),
        ])?;

        CsvWriter::new(file)
            .include_header(true)
            .with_separator(b',')
            .finish(&mut blank_budget)?;

        println!("Populate budget.csv before running Budgy again.");
        Err(PolarsError::ComputeError("Empty budget.csv".into()))?
    }

    let budget = LazyCsvReader::new(path).with_has_header(true).finish()?;
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

    Ok(filtered_budget)
}

fn generate_undefined_categories(statement: LazyFrame) -> PolarsResult<()> {
    println!("===== UNDEFINED CATEGORIES =====");
    let undefined_categories = statement
        .filter(col("New Category").is_null())
        .select([col("Description").unique()])
        .sort(
            ["Description"],
            SortMultipleOptions::new().with_order_descending(false),
        )
        .collect()?;
    println!("{}", undefined_categories);
    Ok(())
}

fn main() -> PolarsResult<()> {
    unsafe {
        set_var("POLARS_FMT_MAX_ROWS", "-1");
        set_var("POLARS_FMT_MAX_COLS", "-1");
        set_var("POLARS_FMT_STR_LEN", "100");
    };

    let username = env::var("USERNAME").unwrap_or_else(|_| String::from("Unknown"));
    let args = args().collect::<Vec<String>>();

    if args.len() > 1 {
        let bank_statement_path = Path::new(&args[1]);

        let lookup = load_lookup(Path::new(&format!("{}_lookup.csv", username)))?
            .with_column(lit(1).alias("Join Key"));

        let mut statement =
            load_statement(bank_statement_path)?.with_column(lit(1).alias("Join Key"));

        let combined = statement.join(
            lookup,
            [col("Join Key")],
            [col("Join Key")],
            JoinType::Inner.into(),
        );
        let matched = combined.filter(col("Description").str().contains(col("Mask"), true));
        statement = matched
            .drop([col("Join Key")])
            .unique(Some(vec!["Nr".to_string()]), UniqueKeepStrategy::First);

        let budget = load_budget(Path::new(&format!("{}_budget.csv", username)))?;

        let summary = statement
            .clone()
            .lazy()
            .filter(col("New Category").is_not_null())
            .left_join(budget, col("New Category"), col("Category"))
            .filter(
                col("Date")
                    .gt_eq(col("Start Date"))
                    .and(col("Date").lt_eq("End Date")),
            )
            .collect()?;

        let result = summary
            .lazy()
            .group_by([col("New Category"), col("Start Date"), col("End Date")])
            .agg([
                col("Money In").fill_null(lit(0.0)).sum().alias("Total In"),
                col("Money Out")
                    .fill_null(lit(0.0))
                    .sum()
                    .alias("Total Out"),
                col("Fee").fill_null(lit(0.0)).sum().alias("Total Fees"),
                (col("Money In").fill_null(lit(0.0)).sum()
                    + col("Money Out").fill_null(lit(0.0)).sum()
                    + col("Fee").fill_null(lit(0.0)).sum())
                .alias("Net Total"),
                (col("Budget Amount") * col("Month Difference"))
                    .max()
                    .alias("Budgetted Amount"),
                ((col("Money In").fill_null(lit(0.0)).sum()
                    + col("Money Out").fill_null(lit(0.0)).sum()
                    + col("Fee").fill_null(lit(0.0)).sum())
                    - (col("Budget Amount") * col("Month Difference")).max())
                .alias("NET BUDGET"),
            ])
            .sort(
                ["Start Date", "New Category"],
                SortMultipleOptions::new().with_order_descending(false),
            );

        println!("{}", result.clone().collect()?);

        let final_result = result
            .group_by(["New Category"])
            .agg([(col("NET BUDGET").sum()).alias("Budget Balance")])
            .sort(
                ["Budget Balance"],
                SortMultipleOptions::new().with_order_descending(false),
            );

        println!("{}", final_result.collect()?);

        generate_undefined_categories(statement)?;
    }

    let mut input = String::new();

    while !["exit", "close", "kill"].contains(&input.as_str()) {
        println!("MENU");

        input = read_line("> ");
    }

    println!("Goodbye!");

    Ok(())
}

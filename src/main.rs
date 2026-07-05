use std::{
    env::set_var,
    io::{Write, stdin, stdout},
    path::Path,
};

use polars::{
    chunked_array::ops::SortMultipleOptions,
    datatypes::DataType,
    error::PolarsResult,
    lazy::{
        dsl::{col, dtype_col, lit, when},
        frame::{LazyCsvReader, LazyFileListReader},
    },
    prelude::NULL,
};

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    stdout().flush().expect("Failed to flush");
    let mut input = String::new();
    stdin()
        .read_line(&mut input)
        .expect("Failed to read string");
    input.trim().to_string()
}

fn main() -> PolarsResult<()> {
    unsafe {
        set_var("POLARS_FMT_MAX_ROWS", "-1");
        set_var("POLARS_FMT_MAX_COLS", "-1");
        set_var("POLARS_FMT_STR_LEN", "100");
    };

    let username = read_line("Enter your username> ");
    let statement_path = read_line("Enter the path to your bank statement: ");
    let bank_statement_path = Path::new(&statement_path);
    let bank_statement_file_name = bank_statement_path
        .file_name()
        .expect("Invalid path")
        .to_str()
        .expect("Failed to extract file name");

    println!("Opening user profile ...");
    let mut lookup = LazyCsvReader::new(format!("{}_lookup.csv", username))
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
    let new_lookup = lookup.rename(&old_names, &new_names, false).collect()?;

    let masks = new_lookup.column("Mask")?.str()?;
    let categories = new_lookup.column("New Category")?.str()?;
    let mut expr = lit(NULL);

    for (opt_mask, opt_category) in masks.into_iter().zip(categories.into_iter()) {
        if let (Some(mask), Some(category)) = (opt_mask, opt_category) {
            expr = when(col("Description").str().contains(lit(mask), true))
                .then(lit(category))
                .otherwise(expr);
        }
    }

    println!("Opening {} ...", bank_statement_file_name);
    let mut data = LazyCsvReader::new(bank_statement_path)
        .with_has_header(true)
        .finish()?;
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

    println!("Filtering {} ...", bank_statement_file_name);
    let filtered_data = new_data.filter(
        col("Money In")
            .is_not_null()
            .or(col("Money Out").is_not_null().or(col("Fee").is_not_null())),
    );

    let joined = filtered_data.with_columns([expr.alias("New Category")]);

    let summary = joined
        .clone()
        .filter(col("New Category").is_not_null())
        .group_by([col("New Category")])
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
        ]);

    let df = summary.clone().collect()?;
    println!("{}", df);

    println!("===== UNDEFINED CATEGORIES =====");
    let undefined_categories = joined
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

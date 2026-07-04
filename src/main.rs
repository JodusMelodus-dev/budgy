use std::{
    env::set_var,
    fs::File,
    io::{Read, Write, stdin, stdout},
    path::{self, Path},
};

use polars::{
    df,
    error::PolarsResult,
    io::{SerWriter, csv::write::CsvWriter},
    lazy::{
        dsl::{col, lit, when},
        frame::{IntoLazy, LazyCsvReader, LazyFileListReader},
    },
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
    let lookup = LazyCsvReader::new(format!("{}_lookup.csv", username))
        .with_has_header(true)
        .finish()?;

    println!("Opening {} ...", bank_statement_file_name);
    let data = LazyCsvReader::new(bank_statement_path)
        .with_has_header(true)
        .finish()?;

    println!("Filtering {} ...", bank_statement_file_name);
    let filtered_data = data.filter(
        col("Money In")
            .is_not_null()
            .or(col("Money Out").is_not_null().or(col("Fee").is_not_null())),
    );

    println!("Joining {} and user profile ...", bank_statement_file_name);
    let joined = filtered_data
        .left_join(lookup, col("Parent Category"), col("Original Category"))
        .select([col("Nr"), col("Parent Category"), col("New Category")]);

    let df = joined.clone().collect()?;
    println!("{}", df);

    println!("===== UNDEFINED CATEGORIES =====");
    let undefined_categories = joined
        .filter(col("New Category").is_null())
        .select([col("Parent Category").unique()])
        .collect()?;
    println!("{}", undefined_categories);

    Ok(())
}

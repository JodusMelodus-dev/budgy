use std::{
    env::set_var,
    fs::File,
    io::{Read, Write, stdin, stdout},
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
    let bank_statement_path = read_line("Enter the path to your bank statement: ");

    let lf = LazyCsvReader::new(bank_statement_path)
        .with_has_header(true)
        .finish()?;

    let lookup = LazyCsvReader::new(format!("{}_lookup.csv", username))
        .with_has_header(true)
        .finish()?;

    let joined = lf
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

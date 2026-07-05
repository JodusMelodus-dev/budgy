use std::{
    env::set_var,
    fs::File,
    io::{Read, Write, stdin, stdout},
    path::{self, Path},
};

use polars::{
    df,
    error::PolarsResult,
    io::{
        SerReader, SerWriter,
        csv::{read::CsvReader, write::CsvWriter},
    },
    lazy::{
        dsl::{Expr, col, lit, when},
        frame::{IntoLazy, LazyCsvReader, LazyFileListReader},
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
        .finish()?
        .collect()?;

    let masks = lookup.column("Mask")?.str()?;
    let categories = lookup.column("New Category")?.str()?;
    let mut expr = lit(NULL);

    for (opt_mask, opt_category) in masks.into_iter().zip(categories.into_iter()) {
        if let (Some(mask), Some(category)) = (opt_mask, opt_category) {
            expr = when(col("Description").str().contains(lit(mask), true))
                .then(lit(category))
                .otherwise(expr);
        }
    }

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

    let joined = filtered_data
        .with_column(expr.alias("New Category"))
        .select([
            col("Nr"),
            col("Description"),
            col("Parent Category"),
            col("New Category"),
        ]);

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

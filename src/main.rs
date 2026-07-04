use std::fs::File;

use polars::{
    df,
    error::PolarsResult,
    io::{SerWriter, csv::write::CsvWriter},
    lazy::{
        dsl::{col, lit, when},
        frame::{IntoLazy, LazyCsvReader, LazyFileListReader},
    },
};

fn main() -> PolarsResult<()> {
    let lf = LazyCsvReader::new("data.csv")
        .with_has_header(true)
        .finish()?;

    let original_categories = vec!["Food", "Communication"];
    let new_categories = vec!["Foooooood", "Comms"];

    let lookup = df![
        "Original Category" => original_categories,
        "New Category" => new_categories,
    ]?
    .lazy();

    let joined = lf
        .left_join(lookup, col("Parent Category"), col("Original Category"))
        .select([col("Nr"), col("Parent Category"), col("New Category")]);

    let df = joined.collect()?;

    println!("{}", df);

    // let processed_lf = lf
    //     .filter(col("Money In").gt(lit(100)))
    //     .with_column((col("Money In") * lit(1.1)).alias("testt"))
    //     .select([
    //         col("Nr"),
    //         col("Posting Date"),
    //         col("Money In"),
    //         col("testt"),
    //     ]);

    // let mut file = File::create("new.csv").expect("Failed to create the file");

    // CsvWriter::new(&mut file)
    //     .include_header(true)
    //     .with_separator(b',')
    //     .finish(&mut df)?;

    Ok(())
}

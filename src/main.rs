use std::fs::File;

use polars::{
    error::PolarsResult,
    io::{SerWriter, csv::write::CsvWriter},
    lazy::{
        dsl::{col, lit},
        frame::{LazyCsvReader, LazyFileListReader},
    },
};

fn main() -> PolarsResult<()> {
    let lf = LazyCsvReader::new("data.csv")
        .with_has_header(true)
        .finish()?;

    let processed_lf = lf
        .filter(col("Money In").gt(lit(100)))
        .with_column((col("Money In") * lit(1.1)).alias("testt"))
        .select([col("Nr"), col("Posting Date"), col("Money In"), col("testt")]);

    let mut df = processed_lf.collect()?;
    println!("{}", df);

    let mut file = File::create("new.csv").expect("Failed to create the file");

    CsvWriter::new(&mut file)
        .include_header(true)
        .with_separator(b',')
        .finish(&mut df)?;

    Ok(())
}

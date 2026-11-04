use std::io::Cursor;

use polars::{
    error::PolarsError,
    frame::DataFrame,
    io::{
        SerReader as _, SerWriter as _,
        csv::{
            read::{CsvParseOptions, CsvReadOptions},
            write::CsvWriter,
        },
    },
    lazy::{
        dsl::col,
        frame::{IntoLazy as _, LazyFrame},
    },
};
use rfd::FileHandle;
use thiserror::Error;

pub enum BudgyFileType {
    Budget,
    Statement,
    Summary,
}

pub struct ImportFile {
    pub file_type: BudgyFileType,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum BudgyFileError {
    #[error("Budget file is empty")]
    Empty,
    #[error("Failed to read CSV data")]
    Polars(#[from] PolarsError),
    #[error("Failed to write file")]
    File(#[from] std::io::Error),
}

fn import_csv_file(bytes: Vec<u8>) -> Result<LazyFrame, BudgyFileError> {
    if bytes.is_empty() {
        return Err(BudgyFileError::Empty);
    }

    Ok(CsvReadOptions::default()
        .with_has_header(true)
        .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
        .into_reader_with_file_handle(Cursor::new(bytes))
        .finish()?
        .lazy())
}

pub fn import_statement(bytes: Vec<u8>) -> Result<LazyFrame, BudgyFileError> {
    let mut lazy_frame = import_csv_file(bytes)?;

    lazy_frame = lazy_frame.filter(
        col("Category")
            .is_not_null()
            .and(col("Parent Category").is_not_null()),
    );

    Ok(lazy_frame)
}

pub fn import_budget(bytes: Vec<u8>) -> Result<LazyFrame, BudgyFileError> {
    import_csv_file(bytes)
}

pub fn import_summary(bytes: Vec<u8>) -> Result<LazyFrame, BudgyFileError> {
    import_csv_file(bytes)
}

pub async fn export_csv(
    file: &FileHandle,
    data_frame: &mut DataFrame,
) -> Result<(), BudgyFileError> {
    let mut buffer = Vec::new();

    CsvWriter::new(Cursor::new(&mut buffer))
        .include_header(true)
        .with_separator(b',')
        .finish(data_frame)?;

    file.write(&buffer).await?;
    Ok(())
}

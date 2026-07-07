use std::{
    env::{self, args, set_var},
    path::Path,
};

use polars::{
    chunked_array::ops::SortMultipleOptions,
    datatypes::{DataType, PlSmallStr},
    error::PolarsResult,
    frame::DataFrame,
    lazy::{
        dsl::{Expr, StrptimeOptions, col, dtype_col, lit, when},
        frame::{LazyCsvReader, LazyFileListReader, LazyFrame},
    },
    prelude::NULL,
};

fn load_lookup(path: &Path) -> PolarsResult<LazyFrame> {
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

fn generate_masks(lookup: DataFrame) -> PolarsResult<Expr> {
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
    Ok(expr)
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

    let args = args().collect::<Vec<String>>();

    let username = env::var("USERNAME").unwrap_or_else(|_| String::from("Unknown"));
    let statement_path = args[1].clone();
    let bank_statement_path = Path::new(&statement_path);

    let lookup = load_lookup(Path::new(&format!("{}_lookup.csv", username)))?.collect()?;
    let mask_expression = generate_masks(lookup)?;

    let statement =
        load_statement(bank_statement_path)?.with_columns([mask_expression.alias("New Category")]);

    let budget = load_budget(Path::new(&format!("{}_budget.csv", username)))?;

    let summary = statement
        .clone()
        .filter(col("New Category").is_not_null())
        .left_join(budget, col("New Category"), col("Category"))
        .filter(
            col("Date")
                .gt_eq(col("Start Date"))
                .and(col("Date").lt_eq("End Date")),
        );

    println!(
        "{}",
        summary
            .clone()
            .select([
                col("Nr"),
                col("Date"),
                col("Description"),
                col("Money In"),
                col("Money Out"),
                col("Fee"),
                col("New Category"),
                col("Budget Amount"),
                col("Month Difference")
            ])
            .collect()?
    );

    let result = summary
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

    Ok(())
}

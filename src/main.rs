mod app;
mod icon;

use std::env;

use crate::{app::Budgy, icon::generate_icon_data};

fn main() {
    unsafe {
        env::set_var("POLARS_FMT_MAX_ROWS", "-1");
        env::set_var("POLARS_FMT_MAX_COLS", "-1");
        env::set_var("POLARS_FMT_STR_LEN", "100");
    };

    let username = env::var("USERNAME").unwrap_or_else(|_| String::from("Unknown"));
    let args = env::args().collect::<Vec<String>>();

    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport = native_options.viewport.with_icon(generate_icon_data());
    eframe::run_native(
        "Budgy",
        native_options,
        Box::new(|_cc| Ok(Box::new(Budgy::new(username, args.get(1).cloned())))),
    )
    .expect("Failed to run GUI");
}

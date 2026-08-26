mod app;
mod icon;

use crate::{app::Budgy, icon::generate_icon_data};
use eframe::APP_KEY;
use std::env;

fn main() {
    unsafe {
        env::set_var("POLARS_FMT_MAX_COLS", "-1");
    };

    let args = env::args().collect::<Vec<String>>();

    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport = native_options.viewport.with_icon(generate_icon_data());
    eframe::run_native(
        "Budgy",
        native_options,
        Box::new(|cc| {
            let config = cc
                .storage
                .and_then(|storage| eframe::get_value(storage, APP_KEY))
                .unwrap_or_default();
            Ok(Box::new(Budgy::new(config, args.get(1).cloned())))
        }),
    )
    .expect("Failed to run GUI");
}

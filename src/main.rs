#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod extra_ui;

use eframe::APP_KEY;
use std::env;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`)

    let args = env::args().collect::<Vec<String>>();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_icon(
            eframe::icon_data::from_png_bytes(&include_bytes!("../assets/favicon-512x512.png")[..])
                .expect("Failed to load icon"),
        ),
        ..Default::default()
    };

    eframe::run_native(
        "Budgy",
        native_options,
        Box::new(|cc| {
            let config = cc
                .storage
                .and_then(|storage| eframe::get_value(storage, APP_KEY))
                .unwrap_or_default();
            Ok(Box::new(app::Budgy::new(config, args.get(1).cloned())))
        }),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    let config = cc
                        .storage
                        .and_then(|storage| eframe::get_value(storage, APP_KEY))
                        .unwrap_or_default();
                    Ok(Box::new(app::Budgy::new(config, None)))
                }),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(()) => {
                    loading_text.remove();
                }
                Err(err) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {err:?}");
                }
            }
        }
    });
}

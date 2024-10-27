#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub mod black_hole;
pub use black_hole::BlackHoleSimulation;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        drag_and_drop_support: true,
        initial_window_size: Some([1280.0, 1024.0].into()),
        ..Default::default()
    };
    eframe::run_native(
        "Black Hole Simulation",
        options,
        Box::new(|cc| Box::new(BlackHoleSimulation::new(cc))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // redirect log messages to the browser console
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "canvas", // The ID of the canvas element in your HTML
                web_options,
                Box::new(|cc| Box::new(BlackHoleSimulation::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}

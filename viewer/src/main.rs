#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod widgets;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "osu! Database Viewer",
        native_options,
        Box::new(|_| Box::new(app::MainApp::default())),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "appCanvas",
                web_options,
                Box::new(|_| Box::new(app::MainApp::default())),
            )
            .await
            .expect("failed to start eframe");
    });
}

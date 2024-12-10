#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod widgets;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    env_logger::init();

    eframe::run_native(
        "osu! Database Viewer",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::new(app::MainApp::default()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    const CANVAS_ID: &'static str = "appCanvas";

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    wasm_bindgen_futures::spawn_local(async {
        // Retrieve canvas from HTML
        let document = web_sys::window()
            .expect("Unable to locate browser window")
            .document()
            .expect("Unable to locate HTML document");

        let canvas = document
            .get_element_by_id(CANVAS_ID)
            .expect(format!("Failed to find HTML canvas element with ID '{CANVAS_ID}'").as_str())
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect(format!("HTML element with ID '{CANVAS_ID}' is not a canvas").as_str());

        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_| Ok(Box::new(app::MainApp::default()))),
            )
            .await
            .expect("failed to start eframe");
    });
}

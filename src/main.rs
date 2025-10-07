use eframe::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

mod ai;
mod gui;

lazy_static! {
    static ref BOARD_E_SIZE: Mutex<f32> = Mutex::new(640.0);
    static ref GRID_SIZE: Mutex<usize> = Mutex::new(40);
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let board_e_size = *BOARD_E_SIZE.lock().unwrap();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([board_e_size, board_e_size]),

        ..Default::default()
    };
    eframe::run_native(
        "gobang-rs",
        options,
        Box::new(|_cc| Ok(Box::new(gui::GobangApp::new()))),
    )
    .unwrap();
    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions {
        ..Default::default()
    };

    wasm_bindgen_futures::spawn_local(async move {
        let window = web_sys::window().expect("No window");

        // web_sys::console::log_1(&window.inner_width().unwrap());

        if window.inner_width().unwrap().as_f64().unwrap() < 640.0 {
            *BOARD_E_SIZE.lock().unwrap() = 320.0;
            *GRID_SIZE.lock().unwrap() = 20;
        }

        let document = window.document().expect("No document");

        let canvas = document
            .get_element_by_id("canvas")
            .expect("Failed to find canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(gui::GobangApp::new()))),
            )
            .await
            .unwrap();
    });
}

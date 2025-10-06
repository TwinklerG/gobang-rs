use eframe::*;

mod ai;
mod gui;

const BOARD_E_SIZE: f32 = 640.0;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([BOARD_E_SIZE, BOARD_E_SIZE]),
        ..Default::default()
    };
    eframe::run_native(
        "五子棋",
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
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("canvas")
            .expect("Failed to find canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        canvas.set_width(BOARD_E_SIZE as u32);
        canvas.set_height(BOARD_E_SIZE as u32);

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

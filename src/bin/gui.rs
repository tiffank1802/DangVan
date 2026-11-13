#[path = "../gui_app.rs"]
mod gui_app;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1120.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Dang Van — fatigue multiaxiale",
        options,
        Box::new(|_creation_context| Ok(Box::new(gui_app::DangVanApp::new()))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    wasm_bindgen_futures::spawn_local(async {
        let canvas = web_sys::window()
            .expect("Fenêtre du navigateur introuvable")
            .document()
            .expect("Document du navigateur introuvable")
            .get_element_by_id("dangvan-canvas")
            .expect("Canvas #dangvan-canvas introuvable")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("L'élément dangvan-canvas n'est pas un canvas HTML");
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_creation_context| Ok(Box::new(gui_app::DangVanApp::new()))),
            )
            .await;
        result.expect("Échec du démarrage de l'interface Dang Van");
    });
}

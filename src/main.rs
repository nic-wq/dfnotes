use eframe::egui;
use crate::app::NotesApp;

pub mod app;
pub mod models;
pub mod markdown;
pub mod file_ops;
pub mod ui;

fn main() -> Result<(), eframe::Error> {
    let icon = load_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(std::sync::Arc::new(icon)),
        ..Default::default()
    };
    
    eframe::run_native(
        "DFNotes - Text Editor",
        options,
        Box::new(|_cc| {
            let mut app = NotesApp::default();
            app.load_config(); 
            Box::new(app)
        }),
    )
}

fn load_icon() -> egui::IconData {
    let icon_bytes = include_bytes!("../assets/dfnotes.png");
    let image = image::load_from_memory(icon_bytes)
        .expect("Failed to load icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();

    egui::IconData {
        rgba,
        width,
        height,
    }
}

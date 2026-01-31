use eframe::egui;
use crate::app::NotesApp;

pub mod app;
pub mod models;
pub mod markdown;
pub mod file_ops;
pub mod ui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([600.0, 400.0]),
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

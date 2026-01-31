use eframe::egui;
use std::time::{Instant, Duration};
use crate::app::NotesApp;
use crate::models::NotificationType;

impl NotesApp {
    pub fn notify(&mut self, msg: &str) {
        self.notifications.push((msg.to_string(), NotificationType::Info, Instant::now()));
    }

    pub fn notify_success(&mut self, msg: &str) {
        self.notifications.push((msg.to_string(), NotificationType::Success, Instant::now()));
    }

    pub fn notify_error(&mut self, msg: &str) {
        self.notifications.push((msg.to_string(), NotificationType::Error, Instant::now()));
    }
    
    pub fn notify_warning(&mut self, msg: &str) {
        self.notifications.push((msg.to_string(), NotificationType::Warning, Instant::now()));
    }
}

pub fn draw_notifications(app: &mut NotesApp, ctx: &egui::Context) {
    app.notifications.retain(|(_, _, time)| time.elapsed() < Duration::from_secs(3));
    
    let mut offset = 10.0;
    for (i, (msg, n_type, _)) in app.notifications.iter().enumerate() {
        let (color, icon) = match n_type {
            NotificationType::Success => (egui::Color32::from_rgb(0, 200, 100), "✅"),
            NotificationType::Error => (egui::Color32::from_rgb(255, 80, 80), "❌"),
            NotificationType::Warning => (egui::Color32::from_rgb(255, 180, 0), "⚠️"),
            NotificationType::Info => (egui::Color32::from_rgb(80, 150, 255), "ℹ️"),
        };

        egui::Window::new(format!("notify_{}", i))
            .title_bar(false)
            .anchor(egui::Align2::RIGHT_TOP, [-10.0, offset])
            .frame(egui::Frame::window(&ctx.style())
                .fill(egui::Color32::from_rgba_premultiplied(30, 30, 30, 240))
                .stroke(egui::Stroke::new(1.0, color))
                .inner_margin(8.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(icon).size(16.0));
                    ui.label(egui::RichText::new(msg).color(egui::Color32::WHITE).strong());
                });
            });
        offset += 45.0;
    }
}

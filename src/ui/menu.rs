use eframe::egui;
use crate::app::NotesApp;

pub fn draw_menu_bar(app: &mut NotesApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("📄 New File").clicked() {
                    app.check_unsaved(crate::models::UnsavedTarget::New);
                    ui.close_menu();
                }
                if ui.button("➕ New Tab (Ctrl+N)").clicked() {
                    app.tabs.push(crate::models::Tab::default());
                    app.active_tab_index = app.tabs.len() - 1;
                    app.save_config(); // Save new tab state
                    ui.close_menu();
                }
                if ui.button("📂 Open File...").clicked() {
                    app.open_file_dialog();
                    ui.close_menu();
                }
                if ui.button("📁 Open Folder...").clicked() {
                    app.open_folder_dialog();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("💾 Save (Ctrl+S)").clicked() {
                    app.save_file();
                    ui.close_menu();
                }
                if ui.button("💾 Save As...").clicked() {
                    app.save_file_as();
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("🚪 Exit").clicked() {
                    app.save_config();
                    std::process::exit(0);
                }
            });
            
            ui.menu_button("View", |ui| {
                if ui.checkbox(&mut app.use_markdown, "Enable Markdown").changed() {
                    app.save_config();
                }
                if ui.checkbox(&mut app.show_explorer, "Show Explorer").changed() {
                    app.save_config();
                }
                ui.separator();
                if ui.checkbox(&mut app.show_hidden, "Show Hidden Files").changed() {
                    app.refresh_folder_files();
                    app.save_config();
                }
                if ui.checkbox(&mut app.show_all_files, "Show All File Types").changed() {
                    app.refresh_folder_files();
                    app.save_config();
                }
                ui.separator();
                if ui.button("🔍 Search (Ctrl+F)").clicked() {
                    app.show_find = !app.show_find;
                    ui.close_menu();
                }
            });

            ui.menu_button("⚙ Settings", |ui| {
                if ui.checkbox(&mut app.real_time_search, "Real-time search").changed() {
                    app.save_config();
                }
            });
        });
    });
}

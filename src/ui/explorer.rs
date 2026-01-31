use eframe::egui;
use std::path::PathBuf;
use crate::app::NotesApp;

pub fn draw_explorer_panel(app: &mut NotesApp, ctx: &egui::Context) {
    egui::SidePanel::left("explorer_panel").resizable(true).show(ctx, |ui| {
        ui.vertical(|ui| {
            ui.heading("📁 Explorer");
            ui.add_space(5.0);

            // Search Bar
            ui.horizontal(|ui| {
                ui.label("🔍");
                if ui.text_edit_singleline(&mut app.search_query).changed() {
                    app.applied_search_query = app.search_query.clone();
                    app.update_search_results();
                }
            });
            
            ui.separator();

            if let Some(root_path) = app.current_folder.clone() {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(root_path.file_name().and_then(|n| n.to_str()).unwrap_or("Root")).strong());
                    if ui.button("📂 Open...").on_hover_text("Change root folder").clicked() {
                        app.open_folder_dialog();
                    }
                });
                
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_tree_recursive(app, ui, &root_path);
                });
            } else {
                ui.centered_and_justified(|ui| {
                    if ui.button("📁 Open Folder").clicked() {
                        app.open_folder_dialog();
                    }
                });
            }
        });
    });
}

fn render_tree_recursive(app: &mut NotesApp, ui: &mut egui::Ui, path: &PathBuf) {
    let mut files_to_show = Vec::new();
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            
            // Respect visibility toggles
            if !app.show_hidden && name.starts_with('.') { continue; }
            
            if p.is_file() && !app.show_all_files {
                let is_text = match p.extension().and_then(|e| e.to_str()) {
                    Some(ext) => {
                        let ext = ext.to_lowercase();
                        matches!(ext.as_str(), "txt" | "md" | "rs" | "js" | "py" | "c" | "cpp" | "h" | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "sh" | "bash" | "go" | "php" | "ts" | "tsx" | "jsx" | "java" | "kt" | "swift")
                    }
                    None => true,
                };
                if !is_text { continue; }
            }
            files_to_show.push(p);
        }
    }
    
    // Sort: directories first, then files
    files_to_show.sort_by(|a, b| if a.is_dir() != b.is_dir() { b.is_dir().cmp(&a.is_dir()) } else { a.file_name().cmp(&b.file_name()) });

    for p in files_to_show {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
        
        if p.is_dir() {
            let is_expanded = app.expanded_folders.contains(&p);
            let icon = if is_expanded { "📂" } else { "📁" };
            
            let header = egui::CollapsingHeader::new(format!("{} {}", icon, name))
                .id_source(&p)
                .default_open(is_expanded);
                
            let response = header.show(ui, |ui| {
                ui.indent(ui.make_persistent_id(&p), |ui| {
                    render_tree_recursive(app, ui, &p);
                });
            });

            if response.header_response.clicked() {
                if app.expanded_folders.contains(&p) {
                    app.expanded_folders.remove(&p);
                } else {
                    app.expanded_folders.insert(p.clone());
                }
            }

            response.header_response.context_menu(|ui| {
                if ui.button("🗑 Delete Folder").clicked() {
                    app.deleting_path = Some(p.clone());
                    ui.close_menu();
                }
            });
        } else {
            let is_active = app.tabs[app.active_tab_index].path.as_ref() == Some(&p);
            let mut label = egui::RichText::new(format!("📝 {}", name));
            if is_active { label = label.strong().color(egui::Color32::from_rgb(100, 200, 255)); }
            
            let btn = ui.selectable_label(is_active, label);
            
            if btn.clicked() {
                app.load_file(p.clone());
            }

            btn.context_menu(|ui| {
                if ui.button("✏ Rename").clicked() {
                    app.renaming_path = Some(p.clone());
                    app.new_name = name.clone();
                    ui.close_menu();
                }
                if ui.button("🗑 Delete").clicked() {
                    app.deleting_path = Some(p.clone());
                    ui.close_menu();
                }
            });
        }
    }
}

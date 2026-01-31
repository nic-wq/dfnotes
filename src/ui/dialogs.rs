use eframe::egui;
use std::fs;
use crate::app::NotesApp;
use crate::models::UnsavedTarget;

impl NotesApp {
    pub fn check_unsaved(&mut self, target: UnsavedTarget) {
        let needs_warning = match &target {
            UnsavedTarget::CloseTab(idx) => self.tabs[*idx].modified,
            UnsavedTarget::Quit => self.tabs.iter().any(|t| t.modified),
            _ => self.current_tab().modified,
        };

        if needs_warning {
            self.unsaved_target = Some(target);
        } else {
            self.execute_target(target);
        }
    }

    pub fn execute_target(&mut self, target: UnsavedTarget) {
        self.execute_target_logic(target);
        self.save_config();
    }

    fn execute_target_logic(&mut self, target: UnsavedTarget) {
        match target {
            UnsavedTarget::New => {
                self.tabs.push(crate::models::Tab::default());
                self.active_tab_index = self.tabs.len() - 1;
            },
            UnsavedTarget::Open(path) => self.load_file(path),
            UnsavedTarget::Rename(path, name) => self.finish_rename(path, name),
            UnsavedTarget::CloseTab(idx) => self.close_tab_safe(idx),
            UnsavedTarget::Quit => {
                self.save_config();
                std::process::exit(0);
            },
        }
    }

    pub fn close_tab_safe(&mut self, idx: usize) {
        self.tabs.remove(idx);
        if self.tabs.is_empty() {
            self.tabs.push(crate::models::Tab::default());
            self.active_tab_index = 0;
        } else if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        }
    }
}

pub fn draw_dialogs(app: &mut NotesApp, ctx: &egui::Context) {
    // 1. Unsaved Changes Dialog
    if let Some(target) = app.unsaved_target.clone() {
        let title = match &target {
            UnsavedTarget::Quit => "⚠️ Quit program?",
            UnsavedTarget::CloseTab(_) => "⚠️ Close tab?",
            _ => "⚠️ Unsaved changes",
        };

        egui::Window::new(title).collapsible(false).resizable(false).show(ctx, |ui| {
            match &target {
                UnsavedTarget::Quit => {
                    ui.label("The following tabs have unsaved changes:");
                    ui.add_space(5.0);
                    egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                        for tab in &app.tabs {
                            if tab.modified {
                                let name = tab.path.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("Untitled");
                                ui.label(format!(" • {}", name));
                            }
                        }
                    });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("💾 Save All and Exit").clicked() {
                            app.save_all_tabs();
                            app.execute_target(target.clone());
                            app.unsaved_target = None;
                        }
                        if ui.button("🗑 Discard All").clicked() {
                            app.execute_target(target.clone());
                            app.unsaved_target = None;
                        }
                        if ui.button("↩ Cancel").clicked() {
                            app.unsaved_target = None;
                        }
                    });
                },
                UnsavedTarget::CloseTab(idx) => {
                    let tab = &app.tabs[*idx];
                    let name = tab.path.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("Untitled");
                    ui.label(format!("The file '{}' has unsaved changes.", name));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("✅ Save and Close").clicked() {
                            // Switch to tab to save (save_file uses current_tab)
                            app.active_tab_index = *idx;
                            app.save_file();
                            app.execute_target(target.clone());
                            app.unsaved_target = None;
                        }
                        if ui.button("❌ Close without saving").clicked() {
                            app.execute_target(target.clone());
                            app.unsaved_target = None;
                        }
                        if ui.button("↩ Cancel").clicked() {
                            app.unsaved_target = None;
                        }
                    });
                },
                _ => {
                    ui.label("Would you like to save changes before continuing?");
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button("✅ Yes (Save)").clicked() {
                            app.save_file();
                            app.execute_target(target.clone());
                            app.unsaved_target = None;
                        }
                        if ui.button("❌ No").clicked() {
                            app.execute_target(target.clone());
                            app.unsaved_target = None;
                        }
                        if ui.button("↩ Cancel").clicked() {
                            app.unsaved_target = None;
                        }
                    });
                }
            }
        });
    }

    // 2. Rename Dialog
    if let Some(path) = app.renaming_path.clone() {
        egui::Window::new("✏ Rename")
            .id(egui::Id::new("rename_window"))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
            ui.label(format!("New name for: {}", path.file_name().unwrap_or_default().to_string_lossy()));
            let res = ui.text_edit_singleline(&mut app.new_name);
            res.request_focus();
            
            ui.horizontal(|ui| {
                if ui.button("✅ Confirm").clicked() || (ui.input(|i| i.key_pressed(egui::Key::Enter)) && res.lost_focus()) {
                    let name = app.new_name.clone();
                    app.check_unsaved(UnsavedTarget::Rename(path.clone(), name));
                    app.renaming_path = None;
                    app.save_config();
                }
                if ui.button("❌ Cancel").clicked() {
                    app.renaming_path = None;
                }
            });
        });
    }

    // 3. Delete Dialog
    if let Some(path) = app.deleting_path.clone() {
        egui::Window::new("🗑 Delete")
            .id(egui::Id::new("delete_window"))
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
            ui.label(format!("Are you sure you want to delete '{}'?", path.file_name().unwrap_or_default().to_string_lossy()));
            ui.horizontal(|ui| {
                if ui.button("🗑 Yes").clicked() {
                    let _ = if path.is_dir() { fs::remove_dir_all(&path) } else { fs::remove_file(&path) };
                    // If deleted file is open in any tab, close it
                    app.tabs.retain(|t| t.path.as_ref() != Some(&path));
                    if app.tabs.is_empty() { app.tabs.push(crate::models::Tab::default()); }
                    if app.active_tab_index >= app.tabs.len() { app.active_tab_index = app.tabs.len() - 1; }
                    
                    app.refresh_folder_files();
                    app.notify_success("Deleted successfully.");
                    app.deleting_path = None;
                    app.save_config();
                }
                if ui.button("❌ No").clicked() {
                    app.deleting_path = None;
                }
            });
        });
    }
}

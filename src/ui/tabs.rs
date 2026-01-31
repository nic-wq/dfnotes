use eframe::egui;
use crate::app::NotesApp;
use crate::models::Tab;

pub fn draw_tab_bar(app: &mut NotesApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        enum TabAction {
            Switch(usize),
            Close(usize),
            Add,
        }
        let mut tab_action = None;

        for (i, tab) in app.tabs.iter().enumerate() {
            let filename = tab.path.as_ref()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled");
            
            let status_icon = if tab.modified { " ●" } else { "" };
            let label = format!("{}{}", filename, status_icon);
            
            ui.scope(|ui| {
                let is_active = i == app.active_tab_index;
                if is_active {
                    ui.visuals_mut().widgets.inactive.bg_fill = egui::Color32::from_rgb(60, 60, 60);
                }

                ui.horizontal(|ui| {
                    let btn = egui::Button::new(label)
                        .min_size(egui::vec2(100.0, 0.0))
                        .fill(if is_active { egui::Color32::from_rgb(60, 60, 60) } else { egui::Color32::TRANSPARENT });
                    
                    if ui.add(btn).on_hover_text("Click to focus").clicked() {
                        tab_action = Some(TabAction::Switch(i));
                    }
                    
                    if ui.button("×").clicked() {
                        tab_action = Some(TabAction::Close(i));
                    }
                });
            });
            ui.add_space(5.0);
        }

        if ui.button("+").on_hover_text("New Tab (Ctrl+N)").clicked() {
            tab_action = Some(TabAction::Add);
        }

        // Executar a ação fora do loop de iteração
        if let Some(action) = tab_action {
            match action {
                TabAction::Switch(i) => {
                    app.active_tab_index = i;
                    app.save_config();
                },
                TabAction::Close(i) => app.check_unsaved(crate::models::UnsavedTarget::CloseTab(i)),
                TabAction::Add => {
                    app.tabs.push(Tab::default());
                    app.active_tab_index = app.tabs.len() - 1;
                    app.save_config();
                },
            }
        }
    });
}

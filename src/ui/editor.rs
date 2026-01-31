use eframe::egui;
use crate::app::NotesApp;
use crate::markdown;

pub fn draw_editor_area(app: &mut NotesApp, ui: &mut egui::Ui) {
    // Barra de Busca (Ctrl+F)
    if app.show_find {
        ui.horizontal(|ui| {
            ui.label("🔍");
            let res = ui.add(
                egui::TextEdit::singleline(&mut app.find_query)
                    .hint_text("Buscar no texto...")
                    .desired_width(200.0)
            );
            if app.show_find { res.request_focus(); }
            if ui.button("✖").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                app.show_find = false;
            }
        });
        ui.separator();
    }

    // Binary/Media detection
    let is_binary = if let Some(path) = &app.current_tab().path {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => {
                let ext = ext.to_lowercase();
                !matches!(ext.as_str(), "txt" | "md" | "rs" | "js" | "py" | "c" | "cpp" | "h" | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "sh" | "bash" | "go" | "php" | "ts" | "tsx" | "jsx" | "java" | "kt" | "swift")
            }
            None => false,
        }
    } else {
        false
    };

    if is_binary {
        ui.centered_and_justified(|ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("📂 Binary or Media File").size(24.0).strong());
                ui.add_space(10.0);
                ui.label("This file type is not supported for text editing.");
                if let Some(path) = &app.current_tab().path {
                    ui.label(egui::RichText::new(path.to_string_lossy()).color(egui::Color32::GRAY));
                }
            });
        });
        return;
    }

    // Focar o editor ao clicar no fundo
    let area_rect = ui.available_rect_before_wrap();
    if ui.interact(area_rect, ui.id().with("bg_click"), egui::Sense::click()).clicked() {
        ui.ctx().memory_mut(|mem| mem.request_focus(ui.id().with("text_editor_main")));
    }

    egui::ScrollArea::vertical()
        .id_source("main_scroll_area")
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                // Numeração
                ui.vertical(|ui| {
                    ui.set_min_width(45.0);
                    ui.style_mut().spacing.item_spacing.y = 2.0;
                    for line_num in 1..=app.current_tab().content.split('\n').count() {
                        ui.add(egui::Label::new(
                            egui::RichText::new(format!("{:>3} ", line_num))
                                .color(egui::Color32::from_rgb(100, 100, 100))
                                .family(egui::FontFamily::Monospace)
                                .size(13.0)
                        ).selectable(false));
                    }
                });

                ui.separator();

                // Texto
                let use_markdown = app.use_markdown;
                let show_find = app.show_find;
                let find_query = app.find_query.to_lowercase();

                let mut layouter = |ui: &egui::Ui, text: &str, _wrap_width: f32| {
                    let mut job = egui::text::LayoutJob::default();
                    let lines: Vec<&str> = text.split('\n').collect();
                    for (i, line) in lines.iter().enumerate() {
                        if show_find && !find_query.is_empty() && line.to_lowercase().contains(&find_query) {
                            markdown::highlight_line_with_search(line, &mut job, &find_query, use_markdown);
                        } else if use_markdown {
                            markdown::highlight_markdown_line(line, &mut job);
                        } else {
                            job.append(line, 0.0, egui::TextFormat {
                                color: egui::Color32::from_rgb(200, 200, 200),
                                font_id: egui::FontId::monospace(14.0),
                                ..Default::default()
                            });
                        }
                        if i < lines.len() - 1 {
                            job.append("\n", 0.0, egui::TextFormat::default());
                        }
                    }
                    ui.ctx().fonts(|f| f.layout_job(job))
                };

                let response = ui.add_sized(
                    ui.available_size(),
                    egui::TextEdit::multiline(&mut app.current_tab_mut().content)
                        .id(ui.id().with("text_editor_main"))
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter)
                        .frame(false)
                );

                if response.changed() { app.current_tab_mut().modified = true; }
            });
        });
}

pub fn draw_status_bar(app: &mut NotesApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            let name = app.current_tab().path.as_ref().and_then(|p| p.file_name()).and_then(|n| n.to_str()).unwrap_or("Untitled");
            ui.label(format!("📄 {}", name));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if app.current_tab().modified { ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "● Modified"); } else { ui.label("Saved"); }
                ui.separator();
                ui.label(format!("📊 {} lines", app.current_tab().content.split('\n').count()));
            });
        });
    });
}

use eframe::egui;
use std::path::PathBuf;
use std::time::Instant;
use std::collections::{HashMap, VecDeque};
use crate::models::{UnsavedTarget, Tab};

pub struct NotesApp {
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
    pub current_folder: Option<PathBuf>,
    pub folder_files: Vec<PathBuf>,
    pub search_query: String,
    pub show_explorer: bool,
    pub use_markdown: bool,
    pub file_contents_cache: HashMap<PathBuf, String>,
    pub renaming_path: Option<PathBuf>,
    pub new_name: String,
    pub deleting_path: Option<PathBuf>,
    pub show_hidden: bool,
    pub show_all_files: bool,
    pub expanded_folders: std::collections::HashSet<PathBuf>,
    pub notifications: Vec<(String, crate::models::NotificationType, Instant)>,
    pub unsaved_target: Option<UnsavedTarget>,
    pub show_find: bool,
    pub find_query: String,
    pub real_time_search: bool,
    pub applied_search_query: String,
    pub filtered_files: Vec<PathBuf>,
    pub cache_order: VecDeque<PathBuf>,
    pub load_sender: crossbeam_channel::Sender<PathBuf>,
    pub load_receiver: crossbeam_channel::Receiver<(PathBuf, String)>,
}

pub const CACHE_LIMIT: usize = 300;

impl Default for NotesApp {
    fn default() -> Self {
        let (s_req, r_req) = crossbeam_channel::unbounded::<PathBuf>();
        let (s_res, r_res) = crossbeam_channel::unbounded::<(PathBuf, String)>();

        // Background loader thread
        std::thread::spawn(move || {
            while let Ok(path) = r_req.recv() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let _ = s_res.send((path, content));
                }
            }
        });

        Self {
            tabs: vec![Tab::default()],
            active_tab_index: 0,
            current_folder: None,
            folder_files: Vec::new(),
            search_query: String::new(),
            show_explorer: true,
            use_markdown: true,
            file_contents_cache: HashMap::new(),
            renaming_path: None,
            new_name: String::new(),
            deleting_path: None,
            show_hidden: false,
            show_all_files: false,
            expanded_folders: std::collections::HashSet::new(),
            notifications: Vec::new(),
            unsaved_target: None,
            show_find: false,
            find_query: String::new(),
            real_time_search: false,
            applied_search_query: String::new(),
            filtered_files: Vec::new(),
            cache_order: VecDeque::new(),
            load_sender: s_req,
            load_receiver: r_res,
        }
    }
}

impl NotesApp {
    pub fn current_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_index]
    }

    pub fn current_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab_index]
    }
}
impl eframe::App for NotesApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Capturar fechamento da janela se houver alterações não salvas
        if ctx.input(|i| i.viewport().close_requested()) {
            self.save_config(); // Save state on close request
            if self.tabs.iter().any(|t| t.modified) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.unsaved_target = Some(UnsavedTarget::Quit);
            }
        }

        // Process background loads
        let mut loaded = false;
        while let Ok((path, content)) = self.load_receiver.try_recv() {
            // Update the cache first
            self.update_cache(path.clone(), content.clone());
            
            // IF any tab points to this path AND is NOT modified, update it
            for tab in &mut self.tabs {
                if tab.path.as_ref() == Some(&path) && !tab.modified {
                    tab.content = content.clone();
                }
            }
            loaded = true;
        }
        if loaded { ctx.request_repaint(); }

        // Shortcuts
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::T))) {
            self.show_explorer = !self.show_explorer;
            self.save_config();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S))) {
            self.save_file();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::F))) {
            self.show_find = !self.show_find;
        }
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::N))) {
            self.tabs.push(Tab::default());
            self.active_tab_index = self.tabs.len() - 1;
            self.save_config();
        }
        if ctx.input_mut(|i| i.consume_shortcut(&egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::W))) {
            if self.tabs.len() > 1 {
                self.tabs.remove(self.active_tab_index);
                if self.active_tab_index >= self.tabs.len() {
                    self.active_tab_index = self.tabs.len() - 1;
                }
            } else {
                self.tabs[0] = Tab::default();
            }
            self.save_config();
        }

        // UI: Notifications
        crate::ui::notifications::draw_notifications(self, ctx);

        // UI: Menu Bar
        crate::ui::menu::draw_menu_bar(self, ctx);
        
        // Tab Bar
        egui::TopBottomPanel::top("tabs_bar").show(ctx, |ui| {
            crate::ui::tabs::draw_tab_bar(self, ui);
        });

        // UI: Side Explorer
        if self.show_explorer {
            crate::ui::explorer::draw_explorer_panel(self, ctx);
        }

        // UI: Status Bar
        crate::ui::editor::draw_status_bar(self, ctx);
        
        // UI: Central Area - EDITOR
        egui::CentralPanel::default().show(ctx, |ui| {
            crate::ui::editor::draw_editor_area(self, ui);
        });

        // UI: Janelas de Diálogo
        crate::ui::dialogs::draw_dialogs(self, ctx);
    }
}

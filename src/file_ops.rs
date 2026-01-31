use std::fs;
use std::path::PathBuf;
use rayon::prelude::*;
use crate::app::{NotesApp, CACHE_LIMIT};
use crate::models::{UnsavedTarget, Tab};

impl NotesApp {
    pub fn finish_rename(&mut self, path: PathBuf, new_name: String) {
        if let Some(parent) = path.parent() {
            let new_path = parent.join(new_name);
            if fs::rename(&path, &new_path).is_ok() {
                // Update ALL tabs using this file
                for tab in &mut self.tabs {
                    if tab.path.as_ref() == Some(&path) {
                        tab.path = Some(new_path.clone());
                    }
                }
                self.refresh_folder_files();
                self.notify_success("Renamed successfully.");
            }
        }
    }

    pub fn load_file(&mut self, path: PathBuf) {
        if let Ok(content) = fs::read_to_string(&path) {
            self.current_tab_mut().content = content.clone();
            self.current_tab_mut().path = Some(path.clone());
            self.current_tab_mut().modified = false;
            // Register in LRU cache
            self.update_cache(path, content);
        }
    }

    pub fn new_file(&mut self) {
        self.tabs.push(Tab::default());
        self.active_tab_index = self.tabs.len() - 1;
    }

    pub fn save_file(&mut self) {
        let (path_opt, content) = {
            let t = self.current_tab();
            (t.path.clone(), t.content.clone())
        };

        if let Some(path) = path_opt {
            if fs::write(&path, &content).is_ok() {
                self.current_tab_mut().modified = false;
                self.update_cache(path, content);
                self.notify_success("File saved.");
            }
        } else if self.current_folder.is_some() {
            self.save_to_folder_quick();
        } else {
            self.save_file_as();
        }
    }

    pub fn save_to_folder_quick(&mut self) {
        if let Some(folder) = &self.current_folder {
            let mut name = String::from("new_note.txt");
            let mut path = folder.join(&name);
            let mut count = 1;
            while path.exists() {
                name = format!("new_note_{}.txt", count);
                path = folder.join(&name);
                count += 1;
            }
            if fs::write(&path, &self.current_tab().content).is_ok() {
                let p = path.clone();
                let c = self.current_tab().content.clone();
                self.current_tab_mut().path = Some(path);
                self.current_tab_mut().modified = false;
                self.update_cache(p, c);
                self.refresh_folder_files();
                self.notify_success("Saved to folder.");
            }
        }
    }

    pub fn update_cache(&mut self, path: PathBuf, content: String) {
        // Atualizar ordem LRU
        self.cache_order.retain(|p| p != &path);
        self.cache_order.push_back(path.clone());
        
        // Se exceder limite, remover mais antigo
        if self.cache_order.len() > CACHE_LIMIT {
            if let Some(oldest) = self.cache_order.pop_front() {
                self.file_contents_cache.remove(&oldest);
            }
        }
        
        self.file_contents_cache.insert(path, content);
        self.update_search_results();
    }

    pub fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() { self.check_unsaved(UnsavedTarget::Open(path)); }
    }

    pub fn open_folder_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.current_folder = Some(path);
            self.refresh_folder_files();
            self.preload_contents();
            self.notify_success("Pasta aberta.");
        }
    }

    pub fn save_all_tabs(&mut self) {
        let tabs_to_save: Vec<(usize, PathBuf, String)> = self.tabs.iter().enumerate()
            .filter(|(_, t)| t.modified && t.path.is_some())
            .map(|(i, t)| (i, t.path.clone().unwrap(), t.content.clone()))
            .collect();

        for (idx, path, content) in tabs_to_save {
            if fs::write(&path, &content).is_ok() {
                self.tabs[idx].modified = false;
                self.update_cache(path, content);
            }
        }
        self.notify_success("Todas as abas salvas.");
    }

    pub fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            let content = self.current_tab().content.clone();
            if fs::write(&path, &content).is_ok() {
                let p = path.clone();
                self.current_tab_mut().path = Some(path);
                self.current_tab_mut().modified = false;
                self.update_cache(p, content);
                self.refresh_folder_files();
                self.notify_success("Salvo com sucesso.");
            }
        }
    }

    pub fn refresh_folder_files(&mut self) {
        if let Some(folder) = &self.current_folder {
            let mut it = Vec::new();
            if let Ok(es) = fs::read_dir(folder) {
                for e in es.flatten() {
                    let path = e.path();
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    
                    // Filter hidden
                    if !self.show_hidden && filename.starts_with('.') {
                        continue;
                    }
                    
                    // Filter non-readable/binary/media
                    if !self.show_all_files && path.is_file() {
                        let is_text = match path.extension().and_then(|e| e.to_str()) {
                            Some(ext) => {
                                let ext = ext.to_lowercase();
                                matches!(ext.as_str(), "txt" | "md" | "rs" | "js" | "py" | "c" | "cpp" | "h" | "html" | "css" | "json" | "toml" | "yaml" | "yml" | "sh" | "bash" | "go" | "php" | "ts" | "tsx" | "jsx" | "java" | "kt" | "swift")
                            }
                            None => true, // Assume files without extension might be text
                        };
                        if !is_text { continue; }
                    }
                    
                    it.push(path);
                }
            }
            it.sort_by(|a, b| if a.is_dir() != b.is_dir() { b.is_dir().cmp(&a.is_dir()) } else { a.file_name().cmp(&b.file_name()) });
            self.folder_files = it;
            self.update_search_results();
        }
    }

    pub fn update_search_results(&mut self) {
        let query = self.applied_search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_files = self.folder_files.clone();
            return;
        }

        let folder_files = &self.folder_files;
        let cache = &self.file_contents_cache;
        let load_s = &self.load_sender;

        // Pesquisa paralela usando Rayon
        self.filtered_files = folder_files.par_iter()
            .filter(|path| {
                let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
                if filename.contains(&query) { return true; }

                if path.is_file() {
                    if let Some(content) = cache.get(*path) {
                        return content.to_lowercase().contains(&query);
                    } else {
                        // Se não está no cache, pedir carregamento em background
                        let _ = load_s.send((*path).clone());
                    }
                }
                false
            })
            .cloned()
            .collect();
    }

    pub fn preload_contents(&mut self) {
        // Agora o preload é progressivo: pede os primeiros 50 arquivos
        for path in self.folder_files.iter().take(50) {
            if path.is_file() && !self.file_contents_cache.contains_key(path) {
                let _ = self.load_sender.send(path.clone());
            }
        }
    }

    pub fn load_config(&mut self) {
        let mut p = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
        } else if let Ok(profile) = std::env::var("USERPROFILE") {
            PathBuf::from(profile)
        } else {
            std::env::current_dir().unwrap_or_default()
        };
        p.push(".dfnotes_config");
        
        if let Ok(json) = fs::read_to_string(p) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&json) {
                // Folder
                if let Some(folder) = config["current_folder"].as_str() {
                    let path = PathBuf::from(folder);
                    if path.exists() {
                        self.current_folder = Some(path);
                        self.refresh_folder_files();
                    }
                }

                // Tabs
                if let Some(tabs_json) = config["tabs"].as_array() {
                    let mut loaded_tabs = Vec::new();
                    for tab_val in tabs_json {
                        if let Ok(tab) = serde_json::from_value::<Tab>(tab_val.clone()) {
                            loaded_tabs.push(tab);
                        }
                    }
                    if !loaded_tabs.is_empty() {
                        self.tabs = loaded_tabs;
                        self.active_tab_index = config["active_tab"].as_u64().unwrap_or(0) as usize;
                        if self.active_tab_index >= self.tabs.len() { self.active_tab_index = 0; }
                    }
                }

                if let Some(real_time) = config["real_time_search"].as_bool() {
                    self.real_time_search = real_time;
                }

                if let Some(markdown) = config["use_markdown"].as_bool() {
                    self.use_markdown = markdown;
                }

                if let Some(explorer) = config["show_explorer"].as_bool() {
                    self.show_explorer = explorer;
                }

                if let Some(hidden) = config["show_hidden"].as_bool() {
                    self.show_hidden = hidden;
                }

                if let Some(all) = config["show_all_files"].as_bool() {
                    self.show_all_files = all;
                }
                
                // Preload for all open tabs
                for tab in &self.tabs {
                    if let Some(path) = &tab.path {
                        let _ = self.load_sender.send(path.clone());
                    }
                }
                self.preload_contents();
            } else {
                eprintln!("Failed to parse config JSON");
            }
        }
    }

    pub fn save_config(&self) {
        let mut p = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
        } else if let Ok(profile) = std::env::var("USERPROFILE") {
            PathBuf::from(profile)
        } else {
            std::env::current_dir().unwrap_or_default()
        };
        p.push(".dfnotes_config");
        
        let config = serde_json::json!({
            "current_folder": self.current_folder.as_ref().map(|p| p.to_string_lossy()),
            "tabs": self.tabs,
            "active_tab": self.active_tab_index,
            "real_time_search": self.real_time_search,
            "use_markdown": self.use_markdown,
            "show_explorer": self.show_explorer,
            "show_hidden": self.show_hidden,
            "show_all_files": self.show_all_files,
        });

        if let Ok(json) = serde_json::to_string(&config) {
            let _ = fs::write(p, json);
        }
    }

    pub fn clear_config(&mut self) {
        let mut p = if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home)
        } else if let Ok(profile) = std::env::var("USERPROFILE") {
            PathBuf::from(profile)
        } else {
            std::env::current_dir().unwrap_or_default()
        };
        p.push(".dfnotes_config");
        let _ = fs::remove_file(p);
        self.notify_warning("Config cleared.");
    }
}


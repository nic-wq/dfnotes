use std::path::PathBuf;

#[derive(Clone)]
pub enum UnsavedTarget {
    New,
    Open(PathBuf),
    Rename(PathBuf, String),
    CloseTab(usize),
    Quit,
}

#[derive(Clone, Copy, PartialEq)]
pub enum NotificationType {
    Success,
    Info,
    Warning,
    Error,
}

#[derive(Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct Tab {
    pub content: String,
    pub path: Option<PathBuf>,
    pub modified: bool,
}

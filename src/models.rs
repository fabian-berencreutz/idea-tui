use serde_derive::{Serialize, Deserialize};
use std::path::PathBuf;
use ratatui::style::Color;

#[derive(PartialEq, Clone, Debug)]
pub enum AppMode {
    MainMenu,
    CategorySelection,
    ProjectSelection,
    InputUrl,
    CloneCategory,
    Favorites,
    Recent,
    ConfirmOpen,
    Help,
    ThemeSelection,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub base_dir: String,
    pub idea_path: String,
    #[serde(default = "default_terminal_cmd")]
    pub terminal_command: String,
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default)]
    pub recent_projects: Vec<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_terminal_cmd() -> String { "kitty --directory".to_string() }
fn default_theme() -> String { "Darcula (default)".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            base_dir: "/home/fabian/dev".to_string(),
            idea_path: "/opt/intellij-idea-ultimate-edition/bin/idea".to_string(),
            terminal_command: default_terminal_cmd(),
            favorites: Vec::new(),
            recent_projects: Vec::new(),
            theme: default_theme(),
        }
    }
}

pub struct Theme {
    pub border: Color,
    pub header_text: Color,
    pub highlight: Color,
    pub confirm_border: Color,
    pub git_branch: Color,
    pub git_clean: Color,
    pub git_dirty: Color,
    pub no_git: Color,
    pub text: Color,
    pub surface: Color,
    pub error: Color,
}

#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub path: PathBuf,
    pub git_branch: Option<String>,
    pub has_changes: bool,
    pub language: Option<String>,
}

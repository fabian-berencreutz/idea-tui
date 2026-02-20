use std::{error::Error, fs, path::PathBuf, process, time::Instant};
use ratatui::widgets::{ListState, TableState};
use crate::models::{AppMode, Config, ProjectInfo};

pub struct App {
    pub mode: AppMode,
    pub previous_mode: Option<AppMode>,
    pub config: Config,
    pub menu_items: Vec<&'static str>,
    pub menu_state: ListState,
    pub categories: Vec<String>,
    pub category_state: ListState,
    pub selected_category: Option<String>,
    pub projects: Vec<ProjectInfo>,
    pub project_state: TableState,
    pub theme_items: Vec<&'static str>,
    pub theme_state: ListState,
    pub input: String,
    pub status_message: Option<(String, Instant)>,
    pub search_query: String,
    pub is_searching: bool,
    pub pending_project: Option<ProjectInfo>,
}

impl App {
    pub fn new(config: Config) -> App {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        let mut project_state = TableState::default();
        project_state.select(Some(0));
        let mut theme_state = ListState::default();
        theme_state.select(Some(0));
        App {
            mode: AppMode::MainMenu,
            previous_mode: None,
            config,
            menu_items: vec!["Favorites", "Recent Projects", "Open Existing Project", "Clone Repository", "Open IntelliJ IDEA", "Choose Theme"],
            menu_state,
            categories: Vec::new(),
            category_state: ListState::default(),
            selected_category: None,
            projects: Vec::new(),
            project_state,
            theme_items: vec![
                "Darcula (default)",
                "Catppuccin Mocha", 
                "Dracula", 
                "Gruvbox", 
                "Nord", 
                "Solarized Dark", 
                "One Dark", 
                "Tokyo Night", 
                "Everforest", 
                "Rose Pine", 
                "Ayu Mirage"
            ],
            theme_state,
            input: String::new(),
            status_message: None,
            search_query: String::new(),
            is_searching: false,
            pending_project: None,
        }
    }

    pub fn save_config(&self) -> Result<(), Box<dyn Error>> {
        confy::store("idea-tui", None, &self.config)?;
        Ok(())
    }

    pub fn add_to_recent(&mut self, path: String) {
        self.config.recent_projects.retain(|x| x != &path);
        self.config.recent_projects.insert(0, path);
        self.config.recent_projects.truncate(10);
        let _ = self.save_config();
    }

    pub fn refresh_current_view(&mut self) {
        match self.mode {
            AppMode::MainMenu | AppMode::ThemeSelection => {}
            AppMode::CategorySelection | AppMode::CloneCategory => self.load_categories(),
            AppMode::ProjectSelection => if let Some(cat) = self.selected_category.clone() { self.load_projects(cat); }
            AppMode::Favorites => self.load_favorites(),
            AppMode::Recent => self.load_recent(),
            _ => {}
        }
        self.status_message = Some(("Status refreshed!".to_string(), Instant::now()));
    }

    pub fn open_terminal(&mut self) -> Result<(), Box<dyn Error>> {
        let query = self.search_query.to_lowercase();
        let filtered: Vec<&ProjectInfo> = self.projects.iter().filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query)).collect();
        if let Some(i) = self.project_state.selected() {
            if i < filtered.len() {
                let path = filtered[i].path.to_str().unwrap_or("");
                let cmd_parts: Vec<&str> = self.config.terminal_command.split_whitespace().collect();
                if !cmd_parts.is_empty() {
                    let mut command = process::Command::new(cmd_parts[0]);
                    for arg in &cmd_parts[1..] { command.arg(arg); }
                    command.arg(path).spawn()?;
                    self.status_message = Some((format!("Opened terminal for {}!", filtered[i].name), Instant::now()));
                }
            }
        }
        Ok(())
    }

    pub fn toggle_favorite(&mut self) {
        let query = self.search_query.to_lowercase();
        let filtered: Vec<&ProjectInfo> = self.projects.iter().filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query)).collect();
        if let Some(i) = self.project_state.selected() {
            if i < filtered.len() {
                let path_str = filtered[i].path.to_str().unwrap_or("").to_string();
                if self.config.favorites.contains(&path_str) {
                    self.config.favorites.retain(|x| x != &path_str);
                    self.status_message = Some((format!("Removed {} from favorites", filtered[i].name), Instant::now()));
                } else {
                    self.config.favorites.push(path_str);
                    self.status_message = Some((format!("Added {} to favorites", filtered[i].name), Instant::now()));
                }
                let _ = self.save_config();
            }
        }
    }

    pub fn load_favorites(&mut self) {
        let mut favs = Vec::new();
        for path_str in &self.config.favorites {
            let path = PathBuf::from(path_str);
            if path.exists() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
                let (branch, changes) = Self::get_git_info(&path);
                let language = Self::detect_language(&path);
                favs.push(ProjectInfo { name, path, git_branch: branch, has_changes: changes, language });
            }
        }
        favs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.projects = favs;
        self.project_state.select(if self.projects.is_empty() { None } else { Some(0) });
        self.selected_category = None;
    }

    pub fn load_recent(&mut self) {
        let mut recent = Vec::new();
        for path_str in &self.config.recent_projects {
            let path = PathBuf::from(path_str);
            if path.exists() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
                let (branch, changes) = Self::get_git_info(&path);
                let language = Self::detect_language(&path);
                recent.push(ProjectInfo { name, path, git_branch: branch, has_changes: changes, language });
            }
        }
        self.projects = recent;
        self.project_state.select(if self.projects.is_empty() { None } else { Some(0) });
        self.selected_category = None;
    }

    pub fn load_categories(&mut self) {
        let mut cats = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.config.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('.') { cats.push(name.to_string()); }
                    }
                }
            }
        }
        cats.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        self.categories = cats;
        self.category_state.select(if self.categories.is_empty() { None } else { Some(0) });
    }

    pub fn get_git_info(path: &PathBuf) -> (Option<String>, bool) {
        if !path.join(".git").exists() { return (None, false); }
        let branch = process::Command::new("git").arg("branch").arg("--show-current").current_dir(path).output().ok().and_then(|out| String::from_utf8(out.stdout).ok()).map(|s| s.trim().to_string());
        let status = process::Command::new("git").arg("status").arg("--porcelain").current_dir(path).output().ok().map(|out| !out.stdout.is_empty()).unwrap_or(false);
        (branch, status)
    }

    pub fn detect_language(path: &PathBuf) -> Option<String> {
        if path.join("Cargo.toml").exists() { return Some("Rust".to_string()); }
        if path.join("pom.xml").exists() || path.join("build.gradle").exists() { return Some("Java".to_string()); }
        if path.join("package.json").exists() { return Some("JS/TS".to_string()); }
        if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() { return Some("Python".to_string()); }
        if path.join("go.mod").exists() { return Some("Go".to_string()); }
        None
    }

    pub fn load_projects(&mut self, category: String) {
        let mut projs = Vec::new();
        let cat_path = PathBuf::from(&self.config.base_dir).join(&category);
        if let Ok(entries) = fs::read_dir(cat_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('.') {
                            let (branch, changes) = Self::get_git_info(&path);
                            let language = Self::detect_language(&path);
                            projs.push(ProjectInfo { name: name.to_string(), path, git_branch: branch, has_changes: changes, language });
                        }
                    }
                }
            }
        }
        projs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.projects = projs;
        self.project_state.select(if self.projects.is_empty() { None } else { Some(0) });
        self.selected_category = Some(category);
    }

    pub fn get_filtered_categories(&self) -> Vec<String> {
        if self.search_query.is_empty() { self.categories.clone() } 
        else { self.categories.iter().filter(|c| c.to_lowercase().contains(&self.search_query.to_lowercase())).cloned().collect() }
    }

    pub fn next(&mut self) {
        match self.mode {
            AppMode::MainMenu => {
                let i = match self.menu_state.selected() { Some(i) => if i >= self.menu_items.len() - 1 { 0 } else { i + 1 }, None => 0 };
                self.menu_state.select(Some(i));
            }
            AppMode::ThemeSelection => {
                let i = match self.theme_state.selected() { Some(i) => if i >= self.theme_items.len() - 1 { 0 } else { i + 1 }, None => 0 };
                self.theme_state.select(Some(i));
            }
            AppMode::CategorySelection | AppMode::CloneCategory => {
                let len = self.get_filtered_categories().len();
                if len == 0 { return; }
                let i = match self.category_state.selected() { Some(i) => if i >= len - 1 { 0 } else { i + 1 }, None => 0 };
                self.category_state.select(Some(i));
            }
            AppMode::ProjectSelection | AppMode::Favorites | AppMode::Recent => {
                let query = self.search_query.to_lowercase();
                let len = self.projects.iter().filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query)).count();
                if len == 0 { return; }
                let i = match self.project_state.selected() { Some(i) => if i >= len - 1 { 0 } else { i + 1 }, None => 0 };
                self.project_state.select(Some(i));
            }
            _ => {}
        }
    }

    pub fn previous(&mut self) {
        match self.mode {
            AppMode::MainMenu => {
                let i = match self.menu_state.selected() { Some(i) => if i == 0 { self.menu_items.len() - 1 } else { i - 1 }, None => 0 };
                self.menu_state.select(Some(i));
            }
            AppMode::ThemeSelection => {
                let i = match self.theme_state.selected() { Some(i) => if i == 0 { self.theme_items.len() - 1 } else { i - 1 }, None => 0 };
                self.theme_state.select(Some(i));
            }
            AppMode::CategorySelection | AppMode::CloneCategory => {
                let len = self.get_filtered_categories().len();
                if len == 0 { return; }
                let i = match self.category_state.selected() { Some(i) => if i == 0 { len - 1 } else { i - 1 }, None => 0 };
                self.category_state.select(Some(i));
            }
            AppMode::ProjectSelection | AppMode::Favorites | AppMode::Recent => {
                let query = self.search_query.to_lowercase();
                let len = self.projects.iter().filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query)).count();
                if len == 0 { return; }
                let i = match self.project_state.selected() { Some(i) => if i == 0 { len - 1 } else { i - 1 }, None => 0 };
                self.project_state.select(Some(i));
            }
            _ => {}
        }
    }

    pub fn on_enter(&mut self) -> Result<bool, Box<dyn Error>> {
        match self.mode {
            AppMode::MainMenu => {
                match self.menu_state.selected() {
                    Some(0) => { self.load_favorites(); self.mode = AppMode::Favorites; }
                    Some(1) => { self.load_recent(); self.mode = AppMode::Recent; }
                    Some(2) => { self.load_categories(); self.mode = AppMode::CategorySelection; }
                    Some(3) => { self.input.clear(); self.mode = AppMode::InputUrl; }
                    Some(4) => {
                        self.pending_project = Some(ProjectInfo { name: "IntelliJ IDEA".to_string(), path: PathBuf::from("IDE"), git_branch: None, has_changes: false, language: None });
                        self.previous_mode = Some(AppMode::MainMenu);
                        self.mode = AppMode::ConfirmOpen;
                    }
                    Some(5) => { self.mode = AppMode::ThemeSelection; }
                    _ => {}
                }
                Ok(false)
            }
            AppMode::ThemeSelection => {
                if let Some(i) = self.theme_state.selected() {
                    self.config.theme = self.theme_items[i].to_string();
                    let _ = self.save_config();
                    self.mode = AppMode::MainMenu;
                }
                Ok(false)
            }
            AppMode::CategorySelection => {
                let filtered = self.get_filtered_categories();
                if let Some(i) = self.category_state.selected() {
                    if i < filtered.len() {
                        let cat = filtered[i].clone();
                        self.load_projects(cat);
                        self.mode = AppMode::ProjectSelection;
                        self.is_searching = false; self.search_query.clear();
                    }
                }
                Ok(false)
            }
            AppMode::ProjectSelection | AppMode::Favorites | AppMode::Recent => {
                let query = self.search_query.to_lowercase();
                let filtered: Vec<&ProjectInfo> = self.projects.iter().filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query)).collect();
                if let Some(i) = self.project_state.selected() {
                    if i < filtered.len() {
                        let proj = filtered[i];
                        self.pending_project = Some(ProjectInfo { name: proj.name.clone(), path: proj.path.clone(), git_branch: None, has_changes: false, language: None });
                        self.previous_mode = Some(self.mode.clone());
                        self.mode = AppMode::ConfirmOpen;
                    }
                }
                Ok(false)
            }
            AppMode::InputUrl => {
                if !self.input.is_empty() { self.load_categories(); self.mode = AppMode::CloneCategory; }
                Ok(false)
            }
            AppMode::CloneCategory => {
                let filtered = self.get_filtered_categories();
                if let Some(i) = self.category_state.selected() {
                    if i < filtered.len() {
                        let cat = filtered[i].clone();
                        self.clone_repo(cat)?;
                        self.is_searching = false; self.search_query.clear();
                    }
                }
                Ok(false)
            }
            AppMode::ConfirmOpen | AppMode::Help => Ok(false),
        }
    }

    pub fn execute_pending_open(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(proj) = self.pending_project.take() {
            if proj.name == "IntelliJ IDEA" {
                process::Command::new(&self.config.idea_path).stdout(process::Stdio::null()).stderr(process::Stdio::null()).spawn()?;
                self.status_message = Some(("Opening IntelliJ IDEA...".to_string(), Instant::now()));
            } else {
                let path_str = proj.path.to_str().unwrap_or("").to_string();
                let name = proj.name.clone();
                self.add_to_recent(path_str.clone());
                process::Command::new(&self.config.idea_path).arg(path_str).stdout(process::Stdio::null()).stderr(process::Stdio::null()).spawn()?;
                self.status_message = Some((format!("Launched {}!", name), Instant::now()));
            }
        }
        self.mode = self.previous_mode.take().unwrap_or(AppMode::MainMenu);
        Ok(())
    }

    pub fn clone_repo(&mut self, category: String) -> Result<(), Box<dyn Error>> {
        let clone_dir = PathBuf::from(&self.config.base_dir).join(&category);
        let url = self.input.clone();
        let project_name = url.split('/').last().and_then(|s| s.strip_suffix(".git").or(Some(s))).unwrap_or("new-project");
        self.status_message = Some((format!("Cloning {}...", project_name), Instant::now()));
        let mut command = process::Command::new("gh");
        command.arg("repo").arg("clone").arg(&url).arg("--").arg("--quiet").current_dir(&clone_dir).stdout(process::Stdio::null()).stderr(process::Stdio::null());
        let status = match command.status() {
            Ok(s) if s.success() => Ok(s),
            _ => process::Command::new("git").arg("clone").arg("--quiet").arg(&url).current_dir(&clone_dir).stdout(process::Stdio::null()).stderr(process::Stdio::null()).status()
        }?;
        if status.success() {
            let project_path = clone_dir.join(project_name);
            self.add_to_recent(project_path.to_str().unwrap_or("").to_string());
            process::Command::new(&self.config.idea_path).arg(project_path.to_str().unwrap_or("")).stdout(process::Stdio::null()).stderr(process::Stdio::null()).spawn()?;
            self.status_message = Some((format!("Cloned and opened {}!", project_name), Instant::now()));
            self.mode = AppMode::MainMenu;
        } else { self.status_message = Some(("Clone failed!".to_string(), Instant::now())); }
        Ok(())
    }

    pub fn go_back(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
        match self.mode {
            AppMode::MainMenu => {}
            AppMode::CategorySelection | AppMode::InputUrl | AppMode::Favorites | AppMode::Recent | AppMode::ThemeSelection => self.mode = AppMode::MainMenu,
            AppMode::ProjectSelection => self.mode = AppMode::CategorySelection,
            AppMode::CloneCategory => self.mode = AppMode::InputUrl,
            AppMode::ConfirmOpen | AppMode::Help => {
                self.mode = self.previous_mode.take().unwrap_or(AppMode::MainMenu);
                self.pending_project = None;
            }
        }
    }
}

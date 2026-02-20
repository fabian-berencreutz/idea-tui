use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Table, Row, Cell, TableState, Clear},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    Frame, Terminal,
    text::{Line, Span},
};
use serde_derive::{Serialize, Deserialize};
use std::{error::Error, io, fs, path::PathBuf, process, time::{Instant, Duration}};

#[derive(PartialEq, Clone)]
enum AppMode {
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
struct Config {
    base_dir: String,
    idea_path: String,
    #[serde(default = "default_terminal_cmd")]
    terminal_command: String,
    #[serde(default)]
    favorites: Vec<String>,
    #[serde(default)]
    recent_projects: Vec<String>,
    #[serde(default = "default_theme")]
    theme: String,
}

fn default_terminal_cmd() -> String { "kitty --directory".to_string() }
fn default_theme() -> String { "Catppuccin Mocha".to_string() }

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

struct Theme {
    border: Color,
    header_text: Color,
    highlight: Color,
    confirm_border: Color,
    git_branch: Color,
    git_clean: Color,
    git_dirty: Color,
    no_git: Color,
    text: Color,
    surface: Color,
    error: Color,
}

fn get_theme(name: &str) -> Theme {
    match name {
        "Dracula" => Theme {
            border: Color::Rgb(189, 147, 249),
            header_text: Color::Rgb(248, 248, 242),
            highlight: Color::Rgb(255, 121, 198),
            confirm_border: Color::Rgb(255, 184, 108),
            git_branch: Color::Rgb(139, 233, 253),
            git_clean: Color::Rgb(80, 250, 123),
            git_dirty: Color::Rgb(241, 250, 140),
            no_git: Color::Rgb(98, 114, 164),
            text: Color::Rgb(248, 248, 242),
            surface: Color::Rgb(68, 71, 90),
            error: Color::Rgb(255, 85, 85),
        },
        "Gruvbox" => Theme {
            border: Color::Rgb(142, 192, 124),
            header_text: Color::Rgb(235, 219, 178),
            highlight: Color::Rgb(131, 165, 152),
            confirm_border: Color::Rgb(250, 189, 47),
            git_branch: Color::Rgb(211, 134, 155),
            git_clean: Color::Rgb(184, 187, 38),
            git_dirty: Color::Rgb(250, 189, 47),
            no_git: Color::Rgb(146, 131, 116),
            text: Color::Rgb(235, 219, 178),
            surface: Color::Rgb(60, 56, 54),
            error: Color::Rgb(204, 36, 29),
        },
        "Nord" => Theme {
            border: Color::Rgb(136, 192, 208),
            header_text: Color::Rgb(236, 239, 244),
            highlight: Color::Rgb(129, 161, 193),
            confirm_border: Color::Rgb(208, 135, 112),
            git_branch: Color::Rgb(180, 142, 173),
            git_clean: Color::Rgb(163, 190, 140),
            git_dirty: Color::Rgb(235, 203, 139),
            no_git: Color::Rgb(76, 86, 106),
            text: Color::Rgb(236, 239, 244),
            surface: Color::Rgb(59, 66, 82),
            error: Color::Rgb(191, 97, 106),
        },
        "Solarized Dark" => Theme {
            border: Color::Rgb(38, 139, 210),
            header_text: Color::Rgb(131, 148, 150),
            highlight: Color::Rgb(181, 137, 0),
            confirm_border: Color::Rgb(203, 75, 22),
            git_branch: Color::Rgb(211, 54, 130),
            git_clean: Color::Rgb(133, 153, 0),
            git_dirty: Color::Rgb(181, 137, 0),
            no_git: Color::Rgb(88, 110, 117),
            text: Color::Rgb(131, 148, 150),
            surface: Color::Rgb(7, 54, 66),
            error: Color::Rgb(220, 50, 47),
        },
        "One Dark" => Theme {
            border: Color::Rgb(97, 175, 239),
            header_text: Color::Rgb(171, 178, 191),
            highlight: Color::Rgb(198, 120, 221),
            confirm_border: Color::Rgb(209, 154, 102),
            git_branch: Color::Rgb(198, 120, 221),
            git_clean: Color::Rgb(152, 195, 121),
            git_dirty: Color::Rgb(229, 192, 123),
            no_git: Color::Rgb(92, 99, 112),
            text: Color::Rgb(171, 178, 191),
            surface: Color::Rgb(40, 44, 52),
            error: Color::Rgb(224, 108, 117),
        },
        "Tokyo Night" => Theme {
            border: Color::Rgb(122, 162, 247),
            header_text: Color::Rgb(169, 177, 214),
            highlight: Color::Rgb(187, 154, 247),
            confirm_border: Color::Rgb(255, 158, 100),
            git_branch: Color::Rgb(158, 206, 106),
            git_clean: Color::Rgb(115, 218, 202),
            git_dirty: Color::Rgb(224, 175, 104),
            no_git: Color::Rgb(86, 95, 137),
            text: Color::Rgb(169, 177, 214),
            surface: Color::Rgb(26, 27, 38),
            error: Color::Rgb(247, 118, 142),
        },
        "Everforest" => Theme {
            border: Color::Rgb(167, 192, 128),
            header_text: Color::Rgb(211, 198, 170),
            highlight: Color::Rgb(127, 187, 179),
            confirm_border: Color::Rgb(230, 152, 117),
            git_branch: Color::Rgb(214, 153, 182),
            git_clean: Color::Rgb(167, 192, 128),
            git_dirty: Color::Rgb(219, 188, 127),
            no_git: Color::Rgb(133, 146, 137),
            text: Color::Rgb(211, 198, 170),
            surface: Color::Rgb(45, 53, 59),
            error: Color::Rgb(230, 126, 128),
        },
        "Rose Pine" => Theme {
            border: Color::Rgb(156, 207, 216),
            header_text: Color::Rgb(224, 222, 244),
            highlight: Color::Rgb(196, 167, 231),
            confirm_border: Color::Rgb(246, 193, 119),
            git_branch: Color::Rgb(235, 188, 186),
            git_clean: Color::Rgb(49, 116, 143),
            git_dirty: Color::Rgb(246, 193, 119),
            no_git: Color::Rgb(110, 106, 134),
            text: Color::Rgb(224, 222, 244),
            surface: Color::Rgb(31, 29, 46),
            error: Color::Rgb(235, 111, 146),
        },
        "Ayu Mirage" => Theme {
            border: Color::Rgb(92, 207, 230),
            header_text: Color::Rgb(204, 202, 194),
            highlight: Color::Rgb(255, 204, 102),
            confirm_border: Color::Rgb(255, 167, 89),
            git_branch: Color::Rgb(212, 191, 255),
            git_clean: Color::Rgb(186, 230, 126),
            git_dirty: Color::Rgb(255, 213, 128),
            no_git: Color::Rgb(112, 122, 140),
            text: Color::Rgb(204, 202, 194),
            surface: Color::Rgb(31, 36, 48),
            error: Color::Rgb(255, 51, 51),
        },
        _ => Theme {
            border: Color::Rgb(148, 226, 213),
            header_text: Color::Rgb(205, 214, 244),
            highlight: Color::Rgb(137, 180, 250),
            confirm_border: Color::Rgb(250, 179, 135),
            git_branch: Color::Rgb(203, 166, 247),
            git_clean: Color::Rgb(166, 227, 161),
            git_dirty: Color::Rgb(249, 226, 175),
            no_git: Color::Rgb(108, 112, 134),
            text: Color::Rgb(205, 214, 244),
            surface: Color::Rgb(49, 50, 68),
            error: Color::Rgb(243, 139, 168),
        },
    }
}

struct ProjectInfo {
    name: String,
    path: PathBuf,
    git_branch: Option<String>,
    has_changes: bool,
    language: Option<String>,
}

struct App {
    mode: AppMode,
    previous_mode: Option<AppMode>,
    config: Config,
    menu_items: Vec<&'static str>,
    menu_state: ListState,
    categories: Vec<String>,
    category_state: ListState,
    selected_category: Option<String>,
    projects: Vec<ProjectInfo>,
    project_state: TableState,
    theme_items: Vec<&'static str>,
    theme_state: ListState,
    input: String,
    status_message: Option<(String, Instant)>,
    search_query: String,
    is_searching: bool,
    pending_project: Option<ProjectInfo>,
}

impl App {
    fn new(config: Config) -> App {
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

    fn save_config(&self) -> Result<(), Box<dyn Error>> {
        confy::store("idea-tui", None, &self.config)?;
        Ok(())
    }

    fn add_to_recent(&mut self, path: String) {
        self.config.recent_projects.retain(|x| x != &path);
        self.config.recent_projects.insert(0, path);
        self.config.recent_projects.truncate(10);
        let _ = self.save_config();
    }

    fn refresh_current_view(&mut self) {
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

    fn open_terminal(&mut self) -> Result<(), Box<dyn Error>> {
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

    fn toggle_favorite(&mut self) {
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

    fn load_favorites(&mut self) {
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

    fn load_recent(&mut self) {
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

    fn load_categories(&mut self) {
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

    fn get_git_info(path: &PathBuf) -> (Option<String>, bool) {
        if !path.join(".git").exists() { return (None, false); }
        let branch = process::Command::new("git").arg("branch").arg("--show-current").current_dir(path).output().ok().and_then(|out| String::from_utf8(out.stdout).ok()).map(|s| s.trim().to_string());
        let status = process::Command::new("git").arg("status").arg("--porcelain").current_dir(path).output().ok().map(|out| !out.stdout.is_empty()).unwrap_or(false);
        (branch, status)
    }

    fn detect_language(path: &PathBuf) -> Option<String> {
        if path.join("Cargo.toml").exists() { return Some("Rust".to_string()); }
        if path.join("pom.xml").exists() || path.join("build.gradle").exists() { return Some("Java".to_string()); }
        if path.join("package.json").exists() { return Some("JS/TS".to_string()); }
        if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() { return Some("Python".to_string()); }
        if path.join("go.mod").exists() { return Some("Go".to_string()); }
        None
    }

    fn load_projects(&mut self, category: String) {
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

    fn get_filtered_categories(&self) -> Vec<String> {
        if self.search_query.is_empty() { self.categories.clone() } 
        else { self.categories.iter().filter(|c| c.to_lowercase().contains(&self.search_query.to_lowercase())).cloned().collect() }
    }

    fn next(&mut self) {
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

    fn previous(&mut self) {
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

    fn on_enter(&mut self) -> Result<bool, Box<dyn Error>> {
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

    fn execute_pending_open(&mut self) -> Result<(), Box<dyn Error>> {
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

    fn clone_repo(&mut self, category: String) -> Result<(), Box<dyn Error>> {
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

    fn go_back(&mut self) {
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

    fn update_status(&mut self) {
        if let Some((_, time)) = self.status_message {
            if time.elapsed() > Duration::from_secs(3) { self.status_message = None; }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg: Config = confy::load("idea-tui", None)?;
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(cfg);
    let res = run_app(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    if let Err(err) = res { println!("{:?}", err); process::exit(1); }
    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<(), Box<dyn Error>> 
where <B as Backend>::Error: 'static {
    loop {
        app.update_status();
        terminal.draw(|f| ui(f, app))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if app.mode == AppMode::ConfirmOpen {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => { app.execute_pending_open()?; }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Backspace => { app.go_back(); }
                        _ => {}
                    }
                } else if app.mode == AppMode::Help {
                    app.go_back();
                } else if app.is_searching {
                    match key.code {
                        KeyCode::Enter => { app.is_searching = false; }
                        KeyCode::Char(c) => { 
                            app.search_query.push(c); 
                            if let AppMode::CategorySelection | AppMode::CloneCategory = app.mode { app.category_state.select(Some(0)); } 
                            else { app.project_state.select(Some(0)); } 
                        }
                        KeyCode::Backspace => { app.search_query.pop(); }
                        KeyCode::Esc => { app.is_searching = false; app.search_query.clear(); }
                        _ => {}
                    }
                } else if app.mode == AppMode::InputUrl {
                    match key.code {
                        KeyCode::Enter => { app.on_enter()?; }
                        KeyCode::Char(c) => { app.input.push(c); }
                        KeyCode::Backspace => { if app.input.is_empty() { app.mode = AppMode::MainMenu; } else { app.input.pop(); } }
                        KeyCode::Esc => { app.mode = AppMode::MainMenu; }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('f') => { app.toggle_favorite(); }
                        KeyCode::Char('t') => { app.open_terminal()?; }
                        KeyCode::Char('r') => { app.refresh_current_view(); }
                        KeyCode::Char('/') => { if app.mode != AppMode::MainMenu && app.mode != AppMode::ThemeSelection { app.is_searching = true; } }
                        KeyCode::Char('?') => { app.previous_mode = Some(app.mode.clone()); app.mode = AppMode::Help; }
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => { app.on_enter()?; },
                        KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => app.go_back(),
                        KeyCode::Esc => { if !app.search_query.is_empty() { app.search_query.clear(); } else { app.go_back(); } }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn dim_background(f: &mut Frame, theme: &Theme) {
    let area = f.area();
    let buffer = f.buffer_mut();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            if let Some(cell) = buffer.cell_mut((x, y)) {
                cell.set_fg(theme.no_git);
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let theme = get_theme(&app.config.theme);
    let chunks = Layout::default().direction(Direction::Vertical).margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref()).split(f.area());

    let title_text = match app.mode {
        AppMode::MainMenu | AppMode::ConfirmOpen | AppMode::Help | AppMode::ThemeSelection => " idea-tui ".to_string(),
        AppMode::CategorySelection => " Select Category ".to_string(),
        AppMode::ProjectSelection => format!(" Projects in {} ", app.selected_category.as_ref().unwrap_or(&"".to_string())),
        AppMode::InputUrl => " Clone Repository: Paste URL ".to_string(),
        AppMode::CloneCategory => " Select Category to Clone into ".to_string(),
        AppMode::Favorites => " Favorite Projects ".to_string(),
        AppMode::Recent => " Recently Opened Projects ".to_string(),
    };
    f.render_widget(Paragraph::new(title_text).style(Style::default().fg(theme.border).add_modifier(Modifier::BOLD)).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border))), chunks[0]);

    match app.mode {
        AppMode::MainMenu | AppMode::ConfirmOpen | AppMode::Help => {
            let items: Vec<ListItem> = app.menu_items.iter().enumerate().map(|(idx, i)| {
                let is_selected = app.menu_state.selected() == Some(idx);
                let style = if is_selected { Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.text) };
                ListItem::new(*i).style(style)
            }).collect();
            f.render_stateful_widget(List::new(items).block(Block::default().title(" Actions ").borders(Borders::ALL).border_style(Style::default().fg(theme.border))).highlight_style(Style::default()).highlight_symbol(Span::styled("> ", Style::default().fg(theme.highlight))), chunks[1], &mut app.menu_state);
        }
        AppMode::ThemeSelection => {
            let items: Vec<ListItem> = app.theme_items.iter().enumerate().map(|(idx, i)| {
                let is_selected = app.theme_state.selected() == Some(idx);
                let style = if is_selected { Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.text) };
                ListItem::new(*i).style(style)
            }).collect();
            f.render_stateful_widget(List::new(items).block(Block::default().title(" Choose Theme ").borders(Borders::ALL).border_style(Style::default().fg(theme.border))).highlight_style(Style::default()).highlight_symbol(Span::styled("> ", Style::default().fg(theme.highlight))), chunks[1], &mut app.theme_state);
        }
        AppMode::CategorySelection | AppMode::CloneCategory => {
            let filtered = app.get_filtered_categories();
            let items: Vec<ListItem> = if filtered.is_empty() {
                vec![ListItem::new("  No results found").style(Style::default().fg(theme.error).add_modifier(Modifier::ITALIC))]
            } else {
                filtered.iter().enumerate().map(|(idx, c)| {
                    let is_selected = app.category_state.selected() == Some(idx);
                    let style = if is_selected { Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.text) };
                    ListItem::new(format!(" {}", c)).style(style)
                }).collect()
            };
            f.render_stateful_widget(List::new(items).block(Block::default().title(" Categories ").borders(Borders::ALL).border_style(Style::default().fg(theme.border))).highlight_style(Style::default()).highlight_symbol(Span::styled("> ", Style::default().fg(theme.highlight))), chunks[1], &mut app.category_state);
        }
        AppMode::ProjectSelection | AppMode::Favorites | AppMode::Recent => {
            let query = app.search_query.to_lowercase();
            let filtered: Vec<&ProjectInfo> = app.projects.iter().filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query)).collect();
            let rows: Vec<Row> = if filtered.is_empty() {
                vec![Row::new(vec![Cell::from("  No results found").style(Style::default().fg(theme.error).add_modifier(Modifier::ITALIC))])]
            } else {
                filtered.iter().enumerate().map(|(idx, p)| {
                    let is_selected = app.project_state.selected() == Some(idx);
                    let name_style = if is_selected { Style::default().fg(theme.highlight).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.text) };
                    
                    let mut name_spans = vec![Span::styled(p.name.clone(), name_style)];
                    if let Some(lang) = &p.language { name_spans.push(Span::styled(format!(" [{}]", lang), Style::default().fg(theme.border).add_modifier(Modifier::ITALIC))); }
                    
                    let git_status = if let Some(branch) = &p.git_branch {
                        let mut spans = vec![Span::styled("", Style::default().fg(theme.border))];
                        if p.has_changes { spans[0] = Span::styled("", Style::default().fg(theme.git_dirty)); }
                        spans.push(Span::styled("  ", Style::default().fg(theme.no_git)));
                        spans.push(Span::styled(branch, Style::default().fg(theme.git_branch)));
                        Line::from(spans)
                    } else { Line::from(vec![Span::styled(" [no git]", Style::default().fg(theme.no_git))]) };
                    let is_fav = app.config.favorites.contains(&p.path.to_str().unwrap_or("").to_string());
                    let fav_cell = Cell::from(" ").style(Style::default().fg(if is_fav { theme.git_dirty } else { theme.surface }));
                    Row::new(vec![Cell::from(Line::from(name_spans)), Cell::from(git_status), fav_cell])
                }).collect()
            };
            let title = match app.mode { AppMode::Favorites => " Favorites ", AppMode::Recent => " Recently Opened ", _ => " Projects " };
            let table = Table::new(rows, [Constraint::Min(30), Constraint::Length(30), Constraint::Length(5)])
                .block(Block::default().title(title).borders(Borders::ALL).border_style(Style::default().fg(theme.border)))
                .highlight_symbol(Span::styled("> ", Style::default().fg(theme.highlight))).row_highlight_style(Style::default().bg(theme.surface));
            f.render_stateful_widget(table, chunks[1], &mut app.project_state);
        }
        AppMode::InputUrl => {
            let content = if app.input.is_empty() { Line::from(vec![Span::styled("Type or paste Git URL here...", Style::default().fg(theme.no_git).add_modifier(Modifier::ITALIC))]) } 
            else { Line::from(vec![Span::styled(&app.input, Style::default().fg(theme.git_dirty))]) };
            f.render_widget(Paragraph::new(content).block(Block::default().borders(Borders::ALL).title(" Git Repository URL ").border_style(Style::default().fg(theme.border))), chunks[1]);
        }
    }

    if app.mode == AppMode::ConfirmOpen || app.mode == AppMode::Help {
        dim_background(f, &theme);
        let area = if app.mode == AppMode::Help { centered_rect(70, 70, f.area()) } else { centered_rect(60, 20, f.area()) };
        f.render_widget(Clear, area);
        if app.mode == AppMode::ConfirmOpen {
            if let Some(proj) = &app.pending_project {
                let block = Block::default().title(" Confirm ").borders(Borders::ALL).border_style(Style::default().fg(theme.confirm_border));
                let text = format!("\nOpen {} in IntelliJ?\n\n(y)es / (n)o", proj.name);
                f.render_widget(Paragraph::new(text).block(block).alignment(Alignment::Center).style(Style::default().fg(theme.header_text)), area);
            }
        } else {
            let block = Block::default().title(" Help & Shortcuts ").borders(Borders::ALL).border_style(Style::default().fg(theme.border));
            let help_rows = vec![
                Row::new(vec![Cell::from("hjkl / Arrows"), Cell::from("Navigate")]),
                Row::new(vec![Cell::from("Enter / l"), Cell::from("Select / Open / Confirm")]),
                Row::new(vec![Cell::from("Backspace / h"), Cell::from("Go Back / Cancel")]),
                Row::new(vec![Cell::from("/"), Cell::from("Search / Filter")]),
                Row::new(vec![Cell::from("f"), Cell::from("Toggle Favorite")]),
                Row::new(vec![Cell::from("t"), Cell::from("Open Quick Terminal")]),
                Row::new(vec![Cell::from("r"), Cell::from("Refresh Git Status")]),
                Row::new(vec![Cell::from("q"), Cell::from("Quit")]),
                Row::new(vec![Cell::from("Esc"), Cell::from("Clear Search / Main Menu")]),
                Row::new(vec![Cell::from("?"), Cell::from("Toggle Help")]),
            ];
            f.render_widget(Table::new(help_rows, [Constraint::Percentage(40), Constraint::Percentage(60)]).block(block).style(Style::default().fg(theme.header_text)), area);
        }
    }

    let footer_text = if app.is_searching { format!("/{} (Press Enter to browse results)", app.search_query) } else if let Some((msg, _)) = &app.status_message { msg.clone() } else {
        match app.mode {
            AppMode::ConfirmOpen => "y: Yes  •  n: No / Cancel".to_string(),
            AppMode::Help => "Press any key to close".to_string(),
            AppMode::ThemeSelection => "Enter: Apply Theme  •  Backspace: Back".to_string(),
            AppMode::MainMenu => "Enter / Right: Select  •  ?: Help  •  q: Quit".to_string(),
            _ => "/: Search  •  r: Refresh  •  t: Terminal  •  f: Favorite  •  Backspace: Back  •  ?: Help".to_string(),
        }
    };
    f.render_widget(Paragraph::new(footer_text).style(if app.status_message.is_some() { Style::default().fg(theme.git_clean).add_modifier(Modifier::BOLD) } else if app.is_searching { Style::default().fg(theme.git_dirty) } else { Style::default().fg(theme.header_text) }).alignment(Alignment::Center), chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)].as_ref()).split(r);
    Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)].as_ref()).split(popup_layout[1])[1]
}

use crate::error::{IdeaError, Result};
use crate::models::{AppMode, Config, ProjectInfo};
use ratatui::widgets::{ListState, TableState};
use std::{fs, path::{PathBuf, Path}, process, time::Instant};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

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
    pub branches: Vec<String>,
    pub branch_state: ListState,
}

impl App {
    pub fn new(config: Config) -> App {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        let mut project_state = TableState::default();
        project_state.select(Some(0));
        let mut theme_state = ListState::default();
        theme_state.select(Some(0));

        let base_path = PathBuf::from(&config.base_dir);
        let (initial_mode, initial_status) = if !base_path.exists() {
            (
                AppMode::ChangeBaseDir,
                Some((
                    format!(
                        "Welcome! Your base_dir '{}' was not found. Please set a valid path.",
                        config.base_dir
                    ),
                    Instant::now(),
                )),
            )
        } else {
            (AppMode::MainMenu, None)
        };

        let mut app = App {
            mode: initial_mode,
            previous_mode: None,
            config: config.clone(),
            menu_items: vec![
                "Favorites",
                "Recent Projects",
                "Open Existing Project",
                "Clone Repository",
                "Open IntelliJ IDEA",
                "Choose Theme",
                "Change Base Directory",
            ],
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
                "Ayu Mirage",
            ],
            theme_state,
            input: config.base_dir.clone(),
            status_message: initial_status,
            search_query: String::new(),
            is_searching: false,
            pending_project: None,
            branches: Vec::new(),
            branch_state: ListState::default(),
        };

        // Still check for IDEA path, but don't block setup for it.
        let idea_path = PathBuf::from(&app.config.idea_path);
        if !idea_path.exists() {
            let in_path = process::Command::new("which")
                .arg(&app.config.idea_path)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if !in_path {
                app.status_message = Some((
                    format!("Warning: idea_path '{}' not found!", app.config.idea_path),
                    Instant::now(),
                ));
            }
        }

        app
    }

    pub fn save_config(&self) -> Result<()> {
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
        self.reload_current_view();
        self.status_message = Some(("Status refreshed!".to_string(), Instant::now()));
    }

    pub fn reload_current_view(&mut self) {
        let current_selection = self.project_state.selected();
        match self.mode {
            AppMode::MainMenu | AppMode::ThemeSelection => {}
            AppMode::CategorySelection | AppMode::CloneCategory => self.load_categories(),
            AppMode::ProjectSelection => {
                if let Some(cat) = self.selected_category.clone() {
                    self.load_projects(cat);
                }
            }
            AppMode::Favorites => self.load_favorites(),
            AppMode::Recent => self.load_recent(),
            _ => {
                // If in a popup mode, reload data based on where we came from
                if let Some(prev) = &self.previous_mode {
                    match prev {
                        AppMode::ProjectSelection => {
                            if let Some(cat) = self.selected_category.clone() {
                                self.load_projects(cat);
                            }
                        }
                        AppMode::Favorites => self.load_favorites(),
                        AppMode::Recent => self.load_recent(),
                        _ => {}
                    }
                }
            }
        }
        // Restore selection after reload if it exists
        if let Some(idx) = current_selection {
            if idx < self.projects.len() {
                self.project_state.select(Some(idx));
            }
        }
    }

    pub fn open_terminal(&mut self) -> Result<()> {
        let query = self.search_query.to_lowercase();
        let filtered: Vec<&ProjectInfo> = self
            .projects
            .iter()
            .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query))
            .collect();
        if let Some(i) = self.project_state.selected() && i < filtered.len() {
            let path = filtered[i].path.to_str().unwrap_or("");
            let cmd_parts: Vec<&str> =
                self.config.terminal_command.split_whitespace().collect();
            if !cmd_parts.is_empty() {
                let mut command = process::Command::new(cmd_parts[0]);
                for arg in &cmd_parts[1..] {
                    command.arg(arg);
                }
                command
                    .arg(path)
                    .spawn()
                    .map_err(|e| IdeaError::Spawn(e.to_string()))?;
                self.status_message = Some((
                    format!("Opened terminal for {}!", filtered[i].name),
                    Instant::now(),
                ));
            }
        }
        Ok(())
    }

    pub fn toggle_favorite(&mut self) {
        let query = self.search_query.to_lowercase();
        let filtered: Vec<&ProjectInfo> = self
            .projects
            .iter()
            .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query))
            .collect();
        if let Some(i) = self.project_state.selected() && i < filtered.len() {
            let path_str = filtered[i].path.to_str().unwrap_or("").to_string();
            if self.config.favorites.contains(&path_str) {
                self.config.favorites.retain(|x| x != &path_str);
                self.status_message = Some((
                    format!("Removed {} from favorites", filtered[i].name),
                    Instant::now(),
                ));
            } else {
                self.config.favorites.push(path_str);
                self.status_message = Some((
                    format!("Added {} to favorites", filtered[i].name),
                    Instant::now(),
                ));
            }
            let _ = self.save_config();
        }
    }

    pub fn load_favorites(&mut self) {
        let mut favs = Vec::new();
        for path_str in &self.config.favorites {
            let path = PathBuf::from(path_str);
            if path.exists() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let (branch, changes) = Self::get_git_info(&path);
                let language = Self::detect_language(&path);
                favs.push(ProjectInfo {
                    name,
                    path,
                    git_branch: branch,
                    has_changes: changes,
                    language,
                });
            }
        }
        favs.sort_by_key(|a| a.name.to_lowercase());
        self.projects = favs;
        self.project_state.select(if self.projects.is_empty() {
            None
        } else {
            Some(0)
        });
        self.selected_category = None;
    }

    pub fn load_recent(&mut self) {
        let mut recent = Vec::new();
        for path_str in &self.config.recent_projects {
            let path = PathBuf::from(path_str);
            if path.exists() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();
                let (branch, changes) = Self::get_git_info(&path);
                let language = Self::detect_language(&path);
                recent.push(ProjectInfo {
                    name,
                    path,
                    git_branch: branch,
                    has_changes: changes,
                    language,
                });
            }
        }
        self.projects = recent;
        self.project_state.select(if self.projects.is_empty() {
            None
        } else {
            Some(0)
        });
        self.selected_category = None;
    }

    pub fn load_categories(&mut self) {
        let mut cats = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.config.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && !name.starts_with('.')
                {
                    cats.push(name.to_string());
                }
            }
        }
        cats.sort_by_key(|a| a.to_lowercase());
        self.categories = cats;
        self.category_state.select(if self.categories.is_empty() {
            None
        } else {
            Some(0)
        });
    }

    pub fn get_git_info(path: &Path) -> (Option<String>, bool) {
        if !path.join(".git").exists() {
            return (None, false);
        }
        let branch = process::Command::new("git")
            .arg("branch")
            .arg("--show-current")
            .current_dir(path)
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .map(|s| s.trim().to_string());
        let status = process::Command::new("git")
            .arg("status")
            .arg("--porcelain")
            .current_dir(path)
            .output()
            .ok()
            .map(|out| !out.stdout.is_empty())
            .unwrap_or(false);
        (branch, status)
    }

    pub fn detect_language(path: &Path) -> Option<String> {
        if path.join("Cargo.toml").exists() {
            return Some("Rust".to_string());
        }
        if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
            return Some("Java".to_string());
        }
        if path.join("package.json").exists() {
            return Some("JS/TS".to_string());
        }
        if path.join("pyproject.toml").exists() || path.join("requirements.txt").exists() {
            return Some("Python".to_string());
        }
        if path.join("go.mod").exists() {
            return Some("Go".to_string());
        }
        None
    }

    pub fn is_project(path: &Path) -> bool {
        path.join(".git").exists() || Self::detect_language(path).is_some()
    }

    pub fn load_projects(&mut self, category: String) {
        let mut projs = Vec::new();
        let cat_path = if category == "." {
            PathBuf::from(&self.config.base_dir)
        } else {
            PathBuf::from(&self.config.base_dir).join(&category)
        };

        // If the category folder itself is a project, include it
        if category != "." && Self::is_project(&cat_path) {
            let (branch, changes) = Self::get_git_info(&cat_path);
            let language = Self::detect_language(&cat_path);
            projs.push(ProjectInfo {
                name: format!(". ({})", category),
                path: cat_path.clone(),
                git_branch: branch,
                has_changes: changes,
                language,
            });
        }

        if let Ok(entries) = fs::read_dir(&cat_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && !name.starts_with('.')
                {
                    // Only add if it's actually a project or if we're in a category
                    if Self::is_project(&path) {
                        let (branch, changes) = Self::get_git_info(&path);
                        let language = Self::detect_language(&path);
                        projs.push(ProjectInfo {
                            name: name.to_string(),
                            path,
                            git_branch: branch,
                            has_changes: changes,
                            language,
                        });
                    }
                }
            }
        }
        projs.sort_by_key(|a| a.name.to_lowercase());
        self.projects = projs;
        self.project_state.select(if self.projects.is_empty() {
            None
        } else {
            Some(0)
        });
        self.selected_category = Some(category);
    }

    pub fn load_branches(&mut self, path: &Path) {
        let output = process::Command::new("git")
            .arg("branch")
            .arg("--format=%(refname:short)")
            .current_dir(path)
            .output();

        if let Ok(out) = output {
            let branches: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            self.branches = branches;
            self.branch_state.select(if self.branches.is_empty() {
                None
            } else {
                Some(0)
            });
        }
    }

    pub fn switch_branch(&mut self, branch: &str, path: &Path) -> Result<()> {
        let status = process::Command::new("git")
            .arg("checkout")
            .arg(branch)
            .current_dir(path)
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .status()
            .map_err(|e| IdeaError::Git(e.to_string()))?;

        if status.success() {
            self.status_message = Some((
                format!("Switched to branch {}!", branch),
                Instant::now(),
            ));
            self.reload_current_view();
            Ok(())
        } else {
            Err(IdeaError::Git(format!("Failed to switch to branch {}", branch)))
        }
    }

    pub fn checkout_only(&mut self) -> Result<()> {
        if let Some(i) = self.branch_state.selected() && i < self.branches.len() {
            let branch = self.branches[i].clone();
            let path = self.pending_project.as_ref().map(|p| p.path.clone());
            if let Some(p) = path {
                self.switch_branch(&branch, &p)?;
                self.go_back();
            }
        }
        Ok(())
    }

    pub fn get_filtered_categories(&self) -> Vec<String> {
        if self.search_query.is_empty() {
            self.categories.clone()
        } else {
            self.categories
                .iter()
                .filter(|c| c.to_lowercase().contains(&self.search_query.to_lowercase()))
                .cloned()
                .collect()
        }
    }

    pub fn next(&mut self) {
        match self.mode {
            AppMode::MainMenu => {
                let i = match self.menu_state.selected() {
                    Some(i) => {
                        if i >= self.menu_items.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.menu_state.select(Some(i));
            }
            AppMode::ThemeSelection => {
                let i = match self.theme_state.selected() {
                    Some(i) => {
                        if i >= self.theme_items.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.theme_state.select(Some(i));
            }
            AppMode::CategorySelection | AppMode::CloneCategory => {
                let len = self.get_filtered_categories().len();
                if len == 0 {
                    return;
                }
                let i = match self.category_state.selected() {
                    Some(i) => {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.category_state.select(Some(i));
            }
            AppMode::ProjectSelection | AppMode::Favorites | AppMode::Recent => {
                let query = self.search_query.to_lowercase();
                let len = self
                    .projects
                    .iter()
                    .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query))
                    .count();
                if len == 0 {
                    return;
                }
                let i = match self.project_state.selected() {
                    Some(i) => {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.project_state.select(Some(i));
            }
            AppMode::BranchSelection => {
                let len = self.branches.len();
                if len == 0 {
                    return;
                }
                let i = match self.branch_state.selected() {
                    Some(i) => {
                        if i >= len - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                self.branch_state.select(Some(i));
            }
            _ => {}
        }
    }

    pub fn previous(&mut self) {
        match self.mode {
            AppMode::MainMenu => {
                let i = match self.menu_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.menu_items.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.menu_state.select(Some(i));
            }
            AppMode::ThemeSelection => {
                let i = match self.theme_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.theme_items.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.theme_state.select(Some(i));
            }
            AppMode::CategorySelection | AppMode::CloneCategory => {
                let len = self.get_filtered_categories().len();
                if len == 0 {
                    return;
                }
                let i = match self.category_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.category_state.select(Some(i));
            }
            AppMode::ProjectSelection | AppMode::Favorites | AppMode::Recent => {
                let query = self.search_query.to_lowercase();
                let len = self
                    .projects
                    .iter()
                    .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query))
                    .count();
                if len == 0 {
                    return;
                }
                let i = match self.project_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.project_state.select(Some(i));
            }
            AppMode::BranchSelection => {
                let len = self.branches.len();
                if len == 0 {
                    return;
                }
                let i = match self.branch_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            len - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                self.branch_state.select(Some(i));
            }
            _ => {}
        }
    }

    pub fn on_enter(&mut self) -> Result<()> {
        match self.mode {
            AppMode::MainMenu => match self.menu_state.selected() {
                Some(0) => {
                    self.load_favorites();
                    self.mode = AppMode::Favorites;
                }
                Some(1) => {
                    self.load_recent();
                    self.mode = AppMode::Recent;
                }
                Some(2) => {
                    self.load_categories();
                    // If everything in base_dir is a project, skip category view
                    let any_direct_projects = self.categories.iter().any(|c| {
                        let path = PathBuf::from(&self.config.base_dir).join(c);
                        Self::is_project(&path)
                    });

                    if any_direct_projects {
                        self.load_projects(".".to_string());
                        self.mode = AppMode::ProjectSelection;
                    } else {
                        self.mode = AppMode::CategorySelection;
                    }
                }
                Some(3) => {
                    self.input.clear();
                    self.mode = AppMode::InputUrl;
                }
                Some(4) => {
                    self.pending_project = Some(ProjectInfo {
                        name: "IntelliJ IDEA".to_string(),
                        path: PathBuf::from("IDE"),
                        git_branch: None,
                        has_changes: false,
                        language: None,
                    });
                    self.previous_mode = Some(AppMode::MainMenu);
                    self.mode = AppMode::ConfirmOpen;
                }
                Some(5) => {
                    self.mode = AppMode::ThemeSelection;
                }
                Some(6) => {
                    self.input = self.config.base_dir.clone();
                    self.mode = AppMode::ChangeBaseDir;
                }
                _ => {}
            },
            AppMode::ThemeSelection => {
                if let Some(i) = self.theme_state.selected() {
                    self.config.theme = self.theme_items[i].to_string();
                    let _ = self.save_config();
                    self.mode = AppMode::MainMenu;
                }
            }
            AppMode::ChangeBaseDir => {
                if !self.input.is_empty() {
                    let new_path = PathBuf::from(&self.input);
                    if new_path.exists() {
                        self.config.base_dir = self.input.clone();
                        let _ = self.save_config();
                        self.status_message = Some((
                            format!("Base directory updated to {}!", self.input),
                            Instant::now(),
                        ));
                        self.mode = AppMode::MainMenu;
                    } else {
                        self.status_message =
                            Some(("Error: Path does not exist!".to_string(), Instant::now()));
                    }
                }
            }
            AppMode::CategorySelection => {
                let filtered = self.get_filtered_categories();
                if let Some(i) = self.category_state.selected() && i < filtered.len() {
                    let cat = filtered[i].clone();
                    self.load_projects(cat);
                    self.mode = AppMode::ProjectSelection;
                    self.is_searching = false;
                    self.search_query.clear();
                }
            }
            AppMode::ProjectSelection | AppMode::Favorites | AppMode::Recent => {
                let query = self.search_query.to_lowercase();
                let filtered: Vec<&ProjectInfo> = self
                    .projects
                    .iter()
                    .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query))
                    .collect();
                if let Some(i) = self.project_state.selected() && i < filtered.len() {
                    let proj = filtered[i];
                    self.pending_project = Some(ProjectInfo {
                        name: proj.name.clone(),
                        path: proj.path.clone(),
                        git_branch: None,
                        has_changes: false,
                        language: None,
                    });
                    self.previous_mode = Some(self.mode.clone());
                    self.mode = AppMode::ConfirmOpen;
                }
            }
            AppMode::InputUrl => {
                if !self.input.is_empty() {
                    self.load_categories();
                    self.mode = AppMode::CloneCategory;
                }
            }
            AppMode::CloneCategory => {
                let filtered = self.get_filtered_categories();
                if let Some(i) = self.category_state.selected() && i < filtered.len() {
                    let cat = filtered[i].clone();
                    self.clone_repo(cat)?;
                    self.is_searching = false;
                    self.search_query.clear();
                }
            }
            AppMode::BranchSelection => {
                if let Some(i) = self.branch_state.selected() && i < self.branches.len() {
                    let branch = self.branches[i].clone();
                    if let Some(proj) = self.pending_project.clone() {
                        self.switch_branch(&branch, &proj.path)?;
                        // After switching branch, proceed to open the project
                        self.mode = AppMode::ConfirmOpen;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn execute_pending_open(&mut self) -> Result<()> {
        if let Some(proj) = self.pending_project.take() {
            if proj.name == "IntelliJ IDEA" {
                self.spawn_process(vec![])?;
                self.status_message =
                    Some(("Opening IntelliJ IDEA...".to_string(), Instant::now()));
            } else {
                let path_str = proj.path.to_str().unwrap_or("").to_string();
                let name = proj.name.clone();
                self.add_to_recent(path_str.clone());
                self.spawn_process(vec![path_str])?;
                self.status_message = Some((format!("Launched {}!", name), Instant::now()));
            }
        }
        self.mode = self.previous_mode.take().unwrap_or(AppMode::MainMenu);
        Ok(())
    }

    pub fn clone_repo(&mut self, category: String) -> Result<()> {
        let clone_dir = PathBuf::from(&self.config.base_dir).join(&category);
        let url = self.input.clone();
        let project_name = url
            .split('/')
            .next_back()
            .and_then(|s| s.strip_suffix(".git").or(Some(s)))
            .unwrap_or("new-project");
        self.status_message = Some((format!("Cloning {}...", project_name), Instant::now()));
        let mut command = process::Command::new("gh");
        command
            .arg("repo")
            .arg("clone")
            .arg(&url)
            .arg("--")
            .arg("--quiet")
            .current_dir(&clone_dir)
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null());
        let status = match command.status() {
            Ok(s) if s.success() => Ok(s),
            _ => process::Command::new("git")
                .arg("clone")
                .arg("--quiet")
                .arg(&url)
                .current_dir(&clone_dir)
                .stdout(process::Stdio::null())
                .stderr(process::Stdio::null())
                .status()
                .map_err(|e| IdeaError::Git(e.to_string())),
        }?;
        if status.success() {
            let project_path = clone_dir.join(project_name);
            let path_str = project_path.to_str().unwrap_or("").to_string();
            self.add_to_recent(path_str.clone());
            self.spawn_process(vec![path_str])?;
            self.status_message = Some((
                format!("Cloned and opened {}!", project_name),
                Instant::now(),
            ));
            self.mode = AppMode::MainMenu;
        } else {
            return Err(IdeaError::CloneFailed(project_name.to_string()));
        }
        Ok(())
    }

    fn spawn_process(&self, args: Vec<String>) -> Result<()> {
        let mut command = process::Command::new(&self.config.idea_path);
        for arg in args {
            command.arg(arg);
        }
        command
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null());

        #[cfg(unix)]
        {
            unsafe {
                command.pre_exec(|| {
                    libc::setsid();
                    Ok(())
                });
            }
        }

        command.spawn().map_err(|e| IdeaError::Spawn(e.to_string()))?;
        Ok(())
    }

    pub fn go_back(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
        match self.mode {
            AppMode::MainMenu => {}
            AppMode::CategorySelection
            | AppMode::InputUrl
            | AppMode::Favorites
            | AppMode::Recent
            | AppMode::ThemeSelection
            | AppMode::ChangeBaseDir => self.mode = AppMode::MainMenu,
            AppMode::ProjectSelection => {
                if self.selected_category == Some(".".to_string()) {
                    self.mode = AppMode::MainMenu;
                } else {
                    self.mode = AppMode::CategorySelection;
                }
            }
            AppMode::CloneCategory => self.mode = AppMode::InputUrl,
            AppMode::ConfirmOpen | AppMode::Help | AppMode::BranchSelection => {
                self.mode = self.previous_mode.take().unwrap_or(AppMode::MainMenu);
                self.pending_project = None;
                self.branches.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_language_rust() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        assert_eq!(
            App::detect_language(&dir.path().to_path_buf()),
            Some("Rust".to_string())
        );
    }

    #[test]
    fn test_detect_language_java_pom() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pom.xml"), "").unwrap();
        assert_eq!(
            App::detect_language(&dir.path().to_path_buf()),
            Some("Java".to_string())
        );
    }

    #[test]
    fn test_detect_language_java_gradle() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("build.gradle"), "").unwrap();
        assert_eq!(
            App::detect_language(&dir.path().to_path_buf()),
            Some("Java".to_string())
        );
    }

    #[test]
    fn test_detect_language_javascript_typescript() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("package.json"), "").unwrap();
        assert_eq!(
            App::detect_language(&dir.path().to_path_buf()),
            Some("JS/TS".to_string())
        );
    }

    #[test]
    fn test_detect_language_python_requirements() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("requirements.txt"), "").unwrap();
        assert_eq!(
            App::detect_language(&dir.path().to_path_buf()),
            Some("Python".to_string())
        );
    }

    #[test]
    fn test_detect_language_python_pyproject() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("pyproject.toml"), "").unwrap();
        assert_eq!(
            App::detect_language(&dir.path().to_path_buf()),
            Some("Python".to_string())
        );
    }

    #[test]
    fn test_detect_language_go() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("go.mod"), "").unwrap();
        assert_eq!(
            App::detect_language(&dir.path().to_path_buf()),
            Some("Go".to_string())
        );
    }

    #[test]
    fn test_detect_language_unknown() {
        let dir = tempdir().unwrap();
        // No language-specific files
        assert_eq!(App::detect_language(&dir.path().to_path_buf()), None);
    }

    #[test]
    fn test_detect_language_empty_dir() {
        let dir = tempdir().unwrap();
        assert_eq!(App::detect_language(&dir.path().to_path_buf()), None);
    }
}

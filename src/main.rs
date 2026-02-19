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

// Catppuccin Mocha Colors
const MOCHA_TEAL: Color = Color::Rgb(148, 226, 213);
const MOCHA_MAUVE: Color = Color::Rgb(203, 166, 247);
const MOCHA_BLUE: Color = Color::Rgb(137, 180, 250);
const MOCHA_PEACH: Color = Color::Rgb(250, 179, 135);
const MOCHA_GREEN: Color = Color::Rgb(166, 227, 161);
const MOCHA_RED: Color = Color::Rgb(243, 139, 168);
const MOCHA_TEXT: Color = Color::Rgb(205, 214, 244);
const MOCHA_OVERLAY: Color = Color::Rgb(108, 112, 134);
const MOCHA_SURFACE: Color = Color::Rgb(49, 50, 68);

#[derive(PartialEq, Clone)]
enum AppMode {
    MainMenu,
    CategorySelection,
    ProjectSelection,
    InputUrl,
    CloneCategory,
    Favorites,
    ConfirmOpen,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    base_dir: String,
    idea_path: String,
    #[serde(default)]
    favorites: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_dir: "/home/fabian/dev".to_string(),
            idea_path: "/opt/intellij-idea-ultimate-edition/bin/idea".to_string(),
            favorites: Vec::new(),
        }
    }
}

struct ProjectInfo {
    name: String,
    path: PathBuf,
    git_branch: Option<String>,
    has_changes: bool,
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
        App {
            mode: AppMode::MainMenu,
            previous_mode: None,
            config,
            menu_items: vec!["Favorites", "Open Existing Project", "Open IntelliJ IDEA", "Clone Repository"],
            menu_state,
            categories: Vec::new(),
            category_state: ListState::default(),
            selected_category: None,
            projects: Vec::new(),
            project_state,
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

    fn toggle_favorite(&mut self) {
        if self.mode == AppMode::ProjectSelection || self.mode == AppMode::Favorites {
            let query = self.search_query.to_lowercase();
            let filtered: Vec<&ProjectInfo> = self.projects.iter()
                .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query))
                .collect();

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
                    if self.mode == AppMode::Favorites { self.load_favorites(); }
                }
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
                favs.push(ProjectInfo { name, path, git_branch: branch, has_changes: changes });
            }
        }
        favs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        self.projects = favs;
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
                            projs.push(ProjectInfo { name: name.to_string(), path, git_branch: branch, has_changes: changes });
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
            AppMode::CategorySelection | AppMode::CloneCategory => {
                let len = self.get_filtered_categories().len();
                if len == 0 { return; }
                let i = match self.category_state.selected() { Some(i) => if i >= len - 1 { 0 } else { i + 1 }, None => 0 };
                self.category_state.select(Some(i));
            }
            AppMode::ProjectSelection | AppMode::Favorites => {
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
            AppMode::CategorySelection | AppMode::CloneCategory => {
                let len = self.get_filtered_categories().len();
                if len == 0 { return; }
                let i = match self.category_state.selected() { Some(i) => if i == 0 { len - 1 } else { i - 1 }, None => 0 };
                self.category_state.select(Some(i));
            }
            AppMode::ProjectSelection | AppMode::Favorites => {
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
                    Some(1) => { self.load_categories(); self.mode = AppMode::CategorySelection; }
                    Some(2) => {
                        self.pending_project = Some(ProjectInfo { name: "IntelliJ IDEA".to_string(), path: PathBuf::from("IDE"), git_branch: None, has_changes: false });
                        self.previous_mode = Some(AppMode::MainMenu);
                        self.mode = AppMode::ConfirmOpen;
                    }
                    Some(3) => { self.input.clear(); self.mode = AppMode::InputUrl; }
                    _ => {}
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
            AppMode::ProjectSelection | AppMode::Favorites => {
                let query = self.search_query.to_lowercase();
                let filtered: Vec<&ProjectInfo> = self.projects.iter().filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query)).collect();
                if let Some(i) = self.project_state.selected() {
                    if i < filtered.len() {
                        let proj = filtered[i];
                        self.pending_project = Some(ProjectInfo { name: proj.name.clone(), path: proj.path.clone(), git_branch: None, has_changes: false });
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
            AppMode::ConfirmOpen => Ok(false),
        }
    }

    fn execute_pending_open(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(proj) = &self.pending_project {
            if proj.name == "IntelliJ IDEA" {
                process::Command::new(&self.config.idea_path).stdout(process::Stdio::null()).stderr(process::Stdio::null()).spawn()?;
                self.status_message = Some(("Opening IntelliJ IDEA...".to_string(), Instant::now()));
            } else {
                process::Command::new(&self.config.idea_path).arg(proj.path.to_str().unwrap_or("")).stdout(process::Stdio::null()).stderr(process::Stdio::null()).spawn()?;
                self.status_message = Some((format!("Launched {}!", proj.name), Instant::now()));
            }
        }
        self.mode = self.previous_mode.take().unwrap_or(AppMode::MainMenu);
        self.pending_project = None;
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
            AppMode::CategorySelection | AppMode::InputUrl | AppMode::Favorites => self.mode = AppMode::MainMenu,
            AppMode::ProjectSelection => self.mode = AppMode::CategorySelection,
            AppMode::CloneCategory => self.mode = AppMode::InputUrl,
            AppMode::ConfirmOpen => {
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
                        KeyCode::Char('/') => { if app.mode != AppMode::MainMenu { app.is_searching = true; } }
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

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default().direction(Direction::Vertical).margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)].as_ref()).split(f.area());

    let title_text = match app.mode {
        AppMode::MainMenu | AppMode::ConfirmOpen => " idea-tui ".to_string(),
        AppMode::CategorySelection => " Select Category ".to_string(),
        AppMode::ProjectSelection => format!(" Projects in {} ", app.selected_category.as_ref().unwrap_or(&"".to_string())),
        AppMode::InputUrl => " Clone Repository: Paste URL ".to_string(),
        AppMode::CloneCategory => " Select Category to Clone into ".to_string(),
        AppMode::Favorites => " Favorite Projects ".to_string(),
    };
    f.render_widget(Paragraph::new(title_text).style(Style::default().fg(MOCHA_TEAL).add_modifier(Modifier::BOLD)).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(MOCHA_TEAL))), chunks[0]);

    match app.mode {
        AppMode::MainMenu | AppMode::ConfirmOpen => {
            let items: Vec<ListItem> = app.menu_items.iter().enumerate().map(|(idx, i)| {
                let is_selected = app.menu_state.selected() == Some(idx);
                let style = if is_selected { Style::default().fg(MOCHA_BLUE).add_modifier(Modifier::BOLD) } else { Style::default().fg(MOCHA_TEXT) };
                ListItem::new(*i).style(style)
            }).collect();
            f.render_stateful_widget(List::new(items).block(Block::default().title(" Actions ").borders(Borders::ALL).border_style(Style::default().fg(MOCHA_TEAL))).highlight_style(Style::default()).highlight_symbol(Span::styled("> ", Style::default().fg(MOCHA_BLUE))), chunks[1], &mut app.menu_state);
        }
        AppMode::CategorySelection | AppMode::CloneCategory => {
            let filtered = app.get_filtered_categories();
            let items: Vec<ListItem> = if filtered.is_empty() {
                vec![ListItem::new("  No results found").style(Style::default().fg(MOCHA_RED).add_modifier(Modifier::ITALIC))]
            } else {
                filtered.iter().enumerate().map(|(idx, c)| {
                    let is_selected = app.category_state.selected() == Some(idx);
                    let style = if is_selected { Style::default().fg(MOCHA_BLUE).add_modifier(Modifier::BOLD) } else { Style::default().fg(MOCHA_TEXT) };
                    ListItem::new(format!(" {}", c)).style(style)
                }).collect()
            };
            f.render_stateful_widget(List::new(items).block(Block::default().title(" Categories ").borders(Borders::ALL).border_style(Style::default().fg(MOCHA_TEAL))).highlight_style(Style::default()).highlight_symbol(Span::styled("> ", Style::default().fg(MOCHA_BLUE))), chunks[1], &mut app.category_state);
        }
        AppMode::ProjectSelection | AppMode::Favorites => {
            let query = app.search_query.to_lowercase();
            let filtered: Vec<&ProjectInfo> = app.projects.iter().filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query)).collect();
            
            let rows: Vec<Row> = if filtered.is_empty() {
                vec![Row::new(vec![Cell::from("  No results found").style(Style::default().fg(MOCHA_RED).add_modifier(Modifier::ITALIC))])]
            } else {
                filtered.iter().enumerate().map(|(idx, p)| {
                    let is_selected = app.project_state.selected() == Some(idx);
                    let name_style = if is_selected { Style::default().fg(MOCHA_BLUE).add_modifier(Modifier::BOLD) } else { Style::default().fg(MOCHA_TEXT) };
                    let name_cell = Cell::from(p.name.clone()).style(name_style);
                    let git_status = if let Some(branch) = &p.git_branch {
                        let mut spans = vec![Span::styled("", Style::default().fg(MOCHA_TEAL))];
                        if p.has_changes { spans[0] = Span::styled("", Style::default().fg(MOCHA_PEACH)); }
                        spans.push(Span::styled("  ", Style::default().fg(MOCHA_OVERLAY)));
                        spans.push(Span::styled(branch, Style::default().fg(MOCHA_MAUVE)));
                        Line::from(spans)
                    } else { Line::from(vec![Span::styled(" [no git]", Style::default().fg(MOCHA_OVERLAY))]) };
                    let is_fav = app.config.favorites.contains(&p.path.to_str().unwrap_or("").to_string());
                    let fav_cell = Cell::from(" ").style(Style::default().fg(if is_fav { MOCHA_PEACH } else { MOCHA_SURFACE }));
                    Row::new(vec![Cell::from(name_cell), Cell::from(git_status), fav_cell])
                }).collect()
            };

                        let title = if app.mode == AppMode::Favorites { " Favorites " } else { " Projects " };
                        let table = Table::new(rows, [Constraint::Min(30), Constraint::Length(30), Constraint::Length(5)])
                            .block(Block::default().title(title).borders(Borders::ALL).border_style(Style::default().fg(MOCHA_TEAL)))
                            .highlight_symbol(Span::styled("> ", Style::default().fg(MOCHA_BLUE)))
                            .row_highlight_style(Style::default().bg(MOCHA_SURFACE));
            f.render_stateful_widget(table, chunks[1], &mut app.project_state);
        }
        AppMode::InputUrl => {
            let content = if app.input.is_empty() { Line::from(vec![Span::styled("Type or paste Git URL here...", Style::default().fg(MOCHA_OVERLAY).add_modifier(Modifier::ITALIC))]) } 
            else { Line::from(vec![Span::styled(&app.input, Style::default().fg(MOCHA_PEACH))]) };
            f.render_widget(Paragraph::new(content).block(Block::default().borders(Borders::ALL).title(" Git Repository URL ").border_style(Style::default().fg(MOCHA_TEAL))), chunks[1]);
        }
    }

    if app.mode == AppMode::ConfirmOpen {
        if let Some(proj) = &app.pending_project {
            let area = centered_rect(60, 20, f.area());
            f.render_widget(Clear, area);
            let block = Block::default().title(" Confirm ").borders(Borders::ALL).border_style(Style::default().fg(MOCHA_PEACH));
            let text = format!("\nOpen {} in IntelliJ?\n\n(y)es / (n)o", proj.name);
            f.render_widget(Paragraph::new(text).block(block).alignment(Alignment::Center).style(Style::default().fg(MOCHA_TEXT)), area);
        }
    }

    let footer_text = if app.is_searching { format!("/{} (Press Enter to browse results)", app.search_query) } else if let Some((msg, _)) = &app.status_message { msg.clone() } else {
        match app.mode {
            AppMode::ConfirmOpen => "y: Yes  •  n: No / Cancel".to_string(),
            AppMode::MainMenu => "Enter / Right: Select  •  q: Quit".to_string(),
            AppMode::CategorySelection => "/: Search  •  Enter / Right: View Projects  •  Backspace: Back  •  q: Quit".to_string(),
            _ => "/: Search  •  Enter / Right: Open  •  f: Favorite  •  Backspace: Back  •  q: Quit".to_string(),
        }
    };
    f.render_widget(Paragraph::new(footer_text).style(if app.status_message.is_some() { Style::default().fg(MOCHA_GREEN).add_modifier(Modifier::BOLD) } else if app.is_searching { Style::default().fg(Color::Yellow) } else { Style::default().fg(MOCHA_TEXT) }).alignment(Alignment::Center), chunks[2]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage((100 - percent_y) / 2), Constraint::Percentage(percent_y), Constraint::Percentage((100 - percent_y) / 2)].as_ref()).split(r);
    Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage((100 - percent_x) / 2), Constraint::Percentage(percent_x), Constraint::Percentage((100 - percent_x) / 2)].as_ref()).split(popup_layout[1])[1]
}

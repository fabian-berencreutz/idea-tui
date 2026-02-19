use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    Frame, Terminal,
    text::{Line, Span},
};
use serde_derive::{Serialize, Deserialize};
use std::{error::Error, io, fs, path::PathBuf, process, time::{Instant, Duration}};

#[derive(PartialEq)]
enum AppMode {
    MainMenu,
    CategorySelection,
    ProjectSelection,
    InputUrl,
    CloneCategory,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    base_dir: String,
    idea_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_dir: "/home/fabian/dev".to_string(),
            idea_path: "/opt/intellij-idea-ultimate-edition/bin/idea".to_string(),
        }
    }
}

struct App {
    mode: AppMode,
    config: Config,
    menu_items: Vec<&'static str>,
    menu_state: ListState,
    categories: Vec<String>,
    category_state: ListState,
    selected_category: Option<String>,
    projects: Vec<String>,
    project_state: ListState,
    input: String,
    status_message: Option<(String, Instant)>,
    search_query: String,
    is_searching: bool,
}

impl App {
    fn new(config: Config) -> App {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        App {
            mode: AppMode::MainMenu,
            config,
            menu_items: vec!["Open Existing Project", "Create New Project", "Clone Repository"],
            menu_state,
            categories: Vec::new(),
            category_state: ListState::default(),
            selected_category: None,
            projects: Vec::new(),
            project_state: ListState::default(),
            input: String::new(),
            status_message: None,
            search_query: String::new(),
            is_searching: false,
        }
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

    fn load_projects(&mut self, category: String) {
        let mut projs = Vec::new();
        let cat_path = PathBuf::from(&self.config.base_dir).join(&category);
        if let Ok(entries) = fs::read_dir(cat_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('.') { projs.push(name.to_string()); }
                    }
                }
            }
        }
        projs.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        self.projects = projs;
        self.project_state.select(if self.projects.is_empty() { None } else { Some(0) });
        self.selected_category = Some(category);
    }

    fn get_filtered_items(&self) -> Vec<String> {
        let items = match self.mode {
            AppMode::CategorySelection | AppMode::CloneCategory => &self.categories,
            AppMode::ProjectSelection => &self.projects,
            _ => return Vec::new(),
        };

        if self.search_query.is_empty() {
            items.clone()
        } else {
            items.iter()
                .filter(|item| item.to_lowercase().contains(&self.search_query.to_lowercase()))
                .cloned()
                .collect()
        }
    }

    fn next(&mut self) {
        match self.mode {
            AppMode::MainMenu => {
                let i = match self.menu_state.selected() { Some(i) => if i >= self.menu_items.len() - 1 { 0 } else { i + 1 }, None => 0 };
                self.menu_state.select(Some(i));
            }
            _ => {
                let filtered_len = self.get_filtered_items().len();
                if filtered_len == 0 { return; }
                match self.mode {
                    AppMode::CategorySelection | AppMode::CloneCategory => {
                        let i = match self.category_state.selected() { Some(i) => if i >= filtered_len - 1 { 0 } else { i + 1 }, None => 0 };
                        self.category_state.select(Some(i));
                    }
                    AppMode::ProjectSelection => {
                        let i = match self.project_state.selected() { Some(i) => if i >= filtered_len - 1 { 0 } else { i + 1 }, None => 0 };
                        self.project_state.select(Some(i));
                    }
                    _ => {}
                }
            }
        }
    }

    fn previous(&mut self) {
        match self.mode {
            AppMode::MainMenu => {
                let i = match self.menu_state.selected() { Some(i) => if i == 0 { self.menu_items.len() - 1 } else { i - 1 }, None => 0 };
                self.menu_state.select(Some(i));
            }
            _ => {
                let filtered_len = self.get_filtered_items().len();
                if filtered_len == 0 { return; }
                match self.mode {
                    AppMode::CategorySelection | AppMode::CloneCategory => {
                        let i = match self.category_state.selected() { Some(i) => if i == 0 { filtered_len - 1 } else { i - 1 }, None => 0 };
                        self.category_state.select(Some(i));
                    }
                    AppMode::ProjectSelection => {
                        let i = match self.project_state.selected() { Some(i) => if i == 0 { filtered_len - 1 } else { i - 1 }, None => 0 };
                        self.project_state.select(Some(i));
                    }
                    _ => {}
                }
            }
        }
    }

    fn on_enter(&mut self) -> Result<bool, Box<dyn Error>> {
        let filtered = self.get_filtered_items();
        match self.mode {
            AppMode::MainMenu => {
                match self.menu_state.selected() {
                    Some(0) => { self.load_categories(); self.mode = AppMode::CategorySelection; }
                    Some(1) => {
                        process::Command::new(&self.config.idea_path)
                            .arg("nosplash").stdout(process::Stdio::null()).stderr(process::Stdio::null()).spawn()?;
                        self.status_message = Some(("Opening Project Wizard...".to_string(), Instant::now()));
                    }
                    Some(2) => { self.input.clear(); self.mode = AppMode::InputUrl; }
                    _ => {}
                }
                Ok(false)
            }
            AppMode::CategorySelection => {
                if let Some(i) = self.category_state.selected() {
                    if i < filtered.len() {
                        let cat = filtered[i].clone();
                        self.load_projects(cat);
                        self.mode = AppMode::ProjectSelection;
                        self.is_searching = false;
                        self.search_query.clear();
                    }
                }
                Ok(false)
            }
            AppMode::ProjectSelection => {
                if let (Some(cat), Some(i)) = (&self.selected_category, self.project_state.selected()) {
                    if i < filtered.len() {
                        let proj = &filtered[i];
                        let path = PathBuf::from(&self.config.base_dir).join(cat).join(proj);
                        process::Command::new(&self.config.idea_path)
                            .arg(path.to_str().unwrap_or("")).stdout(process::Stdio::null()).stderr(process::Stdio::null()).spawn()?;
                        self.status_message = Some((format!("Launched {}!", proj), Instant::now()));
                    }
                }
                Ok(false)
            }
            AppMode::InputUrl => {
                if !self.input.is_empty() {
                    self.load_categories();
                    self.mode = AppMode::CloneCategory;
                }
                Ok(false)
            }
            AppMode::CloneCategory => {
                if let Some(i) = self.category_state.selected() {
                    if i < filtered.len() {
                        let cat = filtered[i].clone();
                        self.clone_repo(cat)?;
                        self.is_searching = false;
                        self.search_query.clear();
                    }
                }
                Ok(false)
            }
        }
    }

    fn clone_repo(&mut self, category: String) -> Result<(), Box<dyn Error>> {
        let clone_dir = PathBuf::from(&self.config.base_dir).join(&category);
        let url = self.input.clone();
        let project_name = url.split('/').last()
            .and_then(|s| s.strip_suffix(".git").or(Some(s)))
            .unwrap_or("new-project");

        self.status_message = Some((format!("Cloning {}...", project_name), Instant::now()));
        let mut command = process::Command::new("gh");
        command.arg("repo").arg("clone").arg(&url).arg("--").arg("--quiet")
            .current_dir(&clone_dir).stdout(process::Stdio::null()).stderr(process::Stdio::null());

        let status = match command.status() {
            Ok(s) if s.success() => Ok(s),
            _ => process::Command::new("git").arg("clone").arg("--quiet").arg(&url)
                .current_dir(&clone_dir).stdout(process::Stdio::null()).stderr(process::Stdio::null()).status()
        }?;

        if status.success() {
            let project_path = clone_dir.join(project_name);
            process::Command::new(&self.config.idea_path)
                .arg(project_path.to_str().unwrap_or(""))
                .stdout(process::Stdio::null()).stderr(process::Stdio::null()).spawn()?;
            self.status_message = Some((format!("Cloned and opened {}!", project_name), Instant::now()));
            self.mode = AppMode::MainMenu;
        } else {
            self.status_message = Some(("Clone failed! (Check your URL or login)".to_string(), Instant::now()));
        }
        Ok(())
    }

    fn go_back(&mut self) {
        self.is_searching = false;
        self.search_query.clear();
        match self.mode {
            AppMode::MainMenu => {}
            AppMode::CategorySelection | AppMode::InputUrl => self.mode = AppMode::MainMenu,
            AppMode::ProjectSelection => self.mode = AppMode::CategorySelection,
            AppMode::CloneCategory => self.mode = AppMode::InputUrl,
        }
    }

    fn update_status(&mut self) {
        if let Some((_, time)) = self.status_message {
            if time.elapsed() > Duration::from_secs(3) {
                self.status_message = None;
            }
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
                if app.is_searching {
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
                        KeyCode::Backspace => { app.input.pop(); }
                        KeyCode::Esc => { app.mode = AppMode::MainMenu; }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('/') => { if app.mode != AppMode::MainMenu { app.is_searching = true; } }
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter => { app.on_enter()?; },
                        KeyCode::Right | KeyCode::Char('l') => { if app.mode != AppMode::MainMenu { app.on_enter()?; } },
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
        AppMode::MainMenu => " idea-tui ".to_string(),
        AppMode::CategorySelection => " Select Category ".to_string(),
        AppMode::ProjectSelection => format!(" Projects in {} ", app.selected_category.as_ref().unwrap_or(&"".to_string())),
        AppMode::InputUrl => " Clone Repository: Paste URL ".to_string(),
        AppMode::CloneCategory => " Select Category to Clone into ".to_string(),
    };
    f.render_widget(Paragraph::new(title_text).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).alignment(Alignment::Center).block(Block::default().borders(Borders::ALL)), chunks[0]);

    match app.mode {
        AppMode::MainMenu => {
            let items: Vec<ListItem> = app.menu_items.iter().map(|i| ListItem::new(*i).style(Style::default().fg(Color::Rgb(255, 255, 255)))).collect();
            f.render_stateful_widget(List::new(items).block(Block::default().title(" Actions ").borders(Borders::ALL)).highlight_style(Style::default().fg(Color::Cyan)).highlight_symbol("> "), chunks[1], &mut app.menu_state);
        }
        AppMode::CategorySelection | AppMode::CloneCategory | AppMode::ProjectSelection => {
            let filtered = app.get_filtered_items();
            let items: Vec<ListItem> = if filtered.is_empty() {
                vec![ListItem::new("  No results found").style(Style::default().fg(Color::Red).add_modifier(Modifier::ITALIC))]
            } else {
                filtered.iter().map(|item| {
                    let display = if app.mode == AppMode::ProjectSelection { item.clone() } else { format!("📁 {}", item) };
                    ListItem::new(display).style(Style::default().fg(Color::Rgb(255, 255, 255)))
                }).collect()
            };
            let title = if app.mode == AppMode::ProjectSelection { " Projects " } else { " Categories " };
            let state = if app.mode == AppMode::ProjectSelection { &mut app.project_state } else { &mut app.category_state };
            f.render_stateful_widget(List::new(items).block(Block::default().title(title).borders(Borders::ALL)).highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).highlight_symbol("> "), chunks[1], state);
        }
        AppMode::InputUrl => {
            let content = if app.input.is_empty() {
                Line::from(vec![
                    Span::styled("Type or paste Git URL here...", Style::default().fg(Color::Rgb(80, 80, 80)).add_modifier(Modifier::ITALIC))
                ])
            } else {
                Line::from(vec![
                    Span::styled(&app.input, Style::default().fg(Color::Yellow))
                ])
            };
            f.render_widget(Paragraph::new(content).block(Block::default().borders(Borders::ALL).title(" Git Repository URL ").border_style(Style::default().fg(Color::Yellow))), chunks[1]);
        }
    }

    let footer_text = if app.is_searching { format!("/{} (Press Enter to browse results)", app.search_query) } else if let Some((msg, _)) = &app.status_message { msg.clone() } else {
        match app.mode {
            AppMode::MainMenu => "Enter: Select  •  q: Quit".to_string(),
            AppMode::CategorySelection => "/: Search  •  Enter / Right: View Projects  •  Backspace: Back  •  q: Quit".to_string(),
            AppMode::ProjectSelection => "/: Search  •  Enter: OPEN IN INTELLIJ  •  Backspace: Back  •  q: Quit".to_string(),
            AppMode::InputUrl => "Type or Paste URL  •  Enter: Continue  •  Esc: Cancel".to_string(),
            AppMode::CloneCategory => "/: Search  •  Select Category  •  Enter: Clone".to_string(),
        }
    };
    f.render_widget(Paragraph::new(footer_text).style(if app.status_message.is_some() { Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) } else if app.is_searching { Style::default().fg(Color::Yellow) } else { Style::default() }).alignment(Alignment::Center), chunks[2]);
}

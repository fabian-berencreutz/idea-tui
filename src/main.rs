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
};
use std::{error::Error, io, fs, path::PathBuf, process, time::{Instant, Duration}};

#[derive(PartialEq)]
enum AppMode {
    MainMenu,
    CategorySelection,
    ProjectSelection,
    InputUrl,
    CloneCategory,
}

struct App {
    mode: AppMode,
    base_dir: PathBuf,
    menu_items: Vec<&'static str>,
    menu_state: ListState,
    categories: Vec<String>,
    category_state: ListState,
    selected_category: Option<String>,
    projects: Vec<String>,
    project_state: ListState,
    input: String,
    status_message: Option<(String, Instant)>,
}

impl App {
    fn new(base_dir: PathBuf) -> App {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        App {
            mode: AppMode::MainMenu,
            base_dir,
            menu_items: vec!["Open Existing Project", "Create New Project", "Clone Repository"],
            menu_state,
            categories: Vec::new(),
            category_state: ListState::default(),
            selected_category: None,
            projects: Vec::new(),
            project_state: ListState::default(),
            input: String::new(),
            status_message: None,
        }
    }

    fn load_categories(&mut self) {
        let mut cats = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.base_dir) {
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
        let cat_path = self.base_dir.join(&category);
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

    fn next(&mut self) {
        match self.mode {
            AppMode::MainMenu => {
                let i = match self.menu_state.selected() { Some(i) => if i >= self.menu_items.len() - 1 { 0 } else { i + 1 }, None => 0 };
                self.menu_state.select(Some(i));
            }
            AppMode::CategorySelection | AppMode::CloneCategory => {
                if self.categories.is_empty() { return; }
                let i = match self.category_state.selected() { Some(i) => if i >= self.categories.len() - 1 { 0 } else { i + 1 }, None => 0 };
                self.category_state.select(Some(i));
            }
            AppMode::ProjectSelection => {
                if self.projects.is_empty() { return; }
                let i = match self.project_state.selected() { Some(i) => if i >= self.projects.len() - 1 { 0 } else { i + 1 }, None => 0 };
                self.project_state.select(Some(i));
            }
            AppMode::InputUrl => {}
        }
    }

    fn previous(&mut self) {
        match self.mode {
            AppMode::MainMenu => {
                let i = match self.menu_state.selected() { Some(i) => if i == 0 { self.menu_items.len() - 1 } else { i - 1 }, None => 0 };
                self.menu_state.select(Some(i));
            }
            AppMode::CategorySelection | AppMode::CloneCategory => {
                if self.categories.is_empty() { return; }
                let i = match self.category_state.selected() { Some(i) => if i == 0 { self.categories.len() - 1 } else { i - 1 }, None => 0 };
                self.category_state.select(Some(i));
            }
            AppMode::ProjectSelection => {
                if self.projects.is_empty() { return; }
                let i = match self.project_state.selected() { Some(i) => if i == 0 { self.projects.len() - 1 } else { i - 1 }, None => 0 };
                self.project_state.select(Some(i));
            }
            AppMode::InputUrl => {}
        }
    }

    fn on_enter(&mut self) -> Result<bool, Box<dyn Error>> {
        match self.mode {
            AppMode::MainMenu => {
                match self.menu_state.selected() {
                    Some(0) => { self.load_categories(); self.mode = AppMode::CategorySelection; }
                    Some(1) => {
                        process::Command::new("/opt/intellij-idea-ultimate-edition/bin/idea")
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
                    let cat = self.categories[i].clone();
                    self.load_projects(cat);
                    self.mode = AppMode::ProjectSelection;
                }
                Ok(false)
            }
            AppMode::ProjectSelection => {
                if let (Some(cat), Some(i)) = (&self.selected_category, self.project_state.selected()) {
                    let proj = &self.projects[i];
                    let path = self.base_dir.join(cat).join(proj);
                    process::Command::new("/opt/intellij-idea-ultimate-edition/bin/idea")
                        .arg(path.to_str().unwrap_or("")).stdout(process::Stdio::null()).stderr(process::Stdio::null()).spawn()?;
                    self.status_message = Some((format!("Launched {}!", proj), Instant::now()));
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
                    let cat = self.categories[i].clone();
                    self.clone_repo(cat)?;
                }
                Ok(false)
            }
        }
    }

    fn clone_repo(&mut self, category: String) -> Result<(), Box<dyn Error>> {
        let clone_dir = self.base_dir.join(&category);
        let url = self.input.clone();
        
        let project_name = url.split('/').last()
            .and_then(|s| s.strip_suffix(".git").or(Some(s)))
            .unwrap_or("new-project");

        self.status_message = Some((format!("Cloning {}...", project_name), Instant::now()));
        
        let mut command = process::Command::new("gh");
        command.arg("repo").arg("clone").arg(&url).arg("--").arg("--quiet")
            .current_dir(&clone_dir)
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null());

        let status = match command.status() {
            Ok(s) if s.success() => Ok(s),
            _ => {
                process::Command::new("git")
                    .arg("clone").arg("--quiet").arg(&url)
                    .current_dir(&clone_dir)
                    .stdout(process::Stdio::null())
                    .stderr(process::Stdio::null())
                    .status()
            }
        }?;

        if status.success() {
            let project_path = clone_dir.join(project_name);
            process::Command::new("/opt/intellij-idea-ultimate-edition/bin/idea")
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
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(PathBuf::from("/home/fabian/dev"));
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
                if app.mode == AppMode::InputUrl {
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
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Enter => { app.on_enter()?; },
                        KeyCode::Right | KeyCode::Char('l') => { 
                            if app.mode != AppMode::MainMenu {
                                app.on_enter()?; 
                            }
                        },
                        KeyCode::Left | KeyCode::Backspace | KeyCode::Char('h') => app.go_back(),
                        KeyCode::Esc => app.go_back(),
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
        AppMode::CategorySelection | AppMode::CloneCategory => {
            let items: Vec<ListItem> = app.categories.iter().map(|c| ListItem::new(format!("📁 {}", c)).style(Style::default().fg(Color::Rgb(255, 255, 255)))).collect();
            f.render_stateful_widget(List::new(items).block(Block::default().title(" Categories ").borders(Borders::ALL)).highlight_style(Style::default().fg(Color::Cyan)).highlight_symbol("> "), chunks[1], &mut app.category_state);
        }
        AppMode::ProjectSelection => {
            let items: Vec<ListItem> = app.projects.iter().map(|p| ListItem::new(p.as_str()).style(Style::default().fg(Color::Rgb(255, 255, 255)))).collect();
            f.render_stateful_widget(List::new(items).block(Block::default().title(" Select Project ").borders(Borders::ALL)).highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)).highlight_symbol("> "), chunks[1], &mut app.project_state);
        }
        AppMode::InputUrl => {
            let input_block = Paragraph::new(app.input.as_str())
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title(" Git Repository URL "));
            f.render_widget(input_block, chunks[1]);
        }
    }

    let footer_text = if let Some((msg, _)) = &app.status_message { msg.clone() } else {
        match app.mode {
            AppMode::MainMenu => "Enter: Select  •  q: Quit".to_string(),
            AppMode::CategorySelection => "Enter / Right: View Projects  •  Backspace: Back  •  q: Quit".to_string(),
            AppMode::ProjectSelection => "Enter: OPEN IN INTELLIJ  •  Backspace: Back  •  q: Quit".to_string(),
            AppMode::InputUrl => "Type or Paste URL  •  Enter: Continue  •  Esc: Cancel".to_string(),
            AppMode::CloneCategory => "Select where to save the project  •  Enter: Clone".to_string(),
        }
    };
    f.render_widget(Paragraph::new(footer_text).style(if app.status_message.is_some() { Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) } else { Style::default() }).alignment(Alignment::Center), chunks[2]);
}

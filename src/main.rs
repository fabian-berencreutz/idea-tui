mod models;
mod theme;
mod app;
mod ui;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{error::Error, io, process, time::Duration};

use crate::app::App;
use crate::models::{AppMode, Config};
use crate::ui::ui;

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
        if let Some((_, time)) = app.status_message {
            if time.elapsed() > Duration::from_secs(3) { app.status_message = None; }
        }
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

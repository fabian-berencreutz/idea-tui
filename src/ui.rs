use crate::app::App;
use crate::models::{AppMode, ProjectInfo, Theme};
use crate::theme::get_theme;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};

pub fn ui(f: &mut Frame, app: &mut App) {
    let theme = get_theme(&app.config.theme);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area());

    let title_text = match app.mode {
        AppMode::MainMenu | AppMode::ConfirmOpen | AppMode::Help | AppMode::ThemeSelection => {
            " idea-tui ".to_string()
        }
        AppMode::CategorySelection => " Select Category ".to_string(),
        AppMode::ProjectSelection => format!(
            " Projects in {} ",
            app.selected_category.as_ref().unwrap_or(&"".to_string())
        ),
        AppMode::InputUrl => " Clone Repository: Paste URL ".to_string(),
        AppMode::CloneCategory => " Select Category to Clone into ".to_string(),
        AppMode::Favorites => " Favorite Projects ".to_string(),
        AppMode::Recent => " Recently Opened Projects ".to_string(),
        AppMode::ChangeBaseDir => " Update Base Directory ".to_string(),
    };
    f.render_widget(
        Paragraph::new(title_text)
            .style(
                Style::default()
                    .fg(theme.border)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            ),
        chunks[0],
    );

    match app.mode {
        AppMode::MainMenu | AppMode::ConfirmOpen | AppMode::Help => {
            let items: Vec<ListItem> = app
                .menu_items
                .iter()
                .enumerate()
                .map(|(idx, i)| {
                    let is_selected = app.menu_state.selected() == Some(idx);
                    let style = if is_selected {
                        Style::default()
                            .fg(theme.highlight)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    };
                    ListItem::new(*i).style(style)
                })
                .collect();
            f.render_stateful_widget(
                List::new(items)
                    .block(
                        Block::default()
                            .title(" Actions ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme.border)),
                    )
                    .highlight_style(Style::default())
                    .highlight_symbol(Span::styled("> ", Style::default().fg(theme.highlight))),
                chunks[1],
                &mut app.menu_state,
            );
        }
        AppMode::ThemeSelection => {
            let items: Vec<ListItem> = app
                .theme_items
                .iter()
                .enumerate()
                .map(|(idx, i)| {
                    let is_selected = app.theme_state.selected() == Some(idx);
                    let style = if is_selected {
                        Style::default()
                            .fg(theme.highlight)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.text)
                    };
                    ListItem::new(*i).style(style)
                })
                .collect();
            f.render_stateful_widget(
                List::new(items)
                    .block(
                        Block::default()
                            .title(" Choose Theme ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme.border)),
                    )
                    .highlight_style(Style::default())
                    .highlight_symbol(Span::styled("> ", Style::default().fg(theme.highlight))),
                chunks[1],
                &mut app.theme_state,
            );
        }
        AppMode::CategorySelection | AppMode::CloneCategory => {
            let filtered = app.get_filtered_categories();
            let items: Vec<ListItem> = if filtered.is_empty() {
                vec![
                    ListItem::new("  No results found").style(
                        Style::default()
                            .fg(theme.error)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]
            } else {
                filtered
                    .iter()
                    .enumerate()
                    .map(|(idx, c)| {
                        let is_selected = app.category_state.selected() == Some(idx);
                        let style = if is_selected {
                            Style::default()
                                .fg(theme.highlight)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.text)
                        };
                        ListItem::new(format!(" {}", c)).style(style)
                    })
                    .collect()
            };
            f.render_stateful_widget(
                List::new(items)
                    .block(
                        Block::default()
                            .title(" Categories ")
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(theme.border)),
                    )
                    .highlight_style(Style::default())
                    .highlight_symbol(Span::styled("> ", Style::default().fg(theme.highlight))),
                chunks[1],
                &mut app.category_state,
            );
        }
        AppMode::ProjectSelection | AppMode::Favorites | AppMode::Recent => {
            let query = app.search_query.to_lowercase();
            let filtered: Vec<&ProjectInfo> = app
                .projects
                .iter()
                .filter(|p| query.is_empty() || p.name.to_lowercase().contains(&query))
                .collect();
            let rows: Vec<Row> = if filtered.is_empty() {
                vec![Row::new(vec![
                    Cell::from("  No results found").style(
                        Style::default()
                            .fg(theme.error)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ])]
            } else {
                filtered
                    .iter()
                    .enumerate()
                    .map(|(idx, p)| {
                        let is_selected = app.project_state.selected() == Some(idx);
                        let name_style = if is_selected {
                            Style::default()
                                .fg(theme.highlight)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.text)
                        };

                        let mut name_spans = vec![Span::styled(p.name.clone(), name_style)];
                        if let Some(lang) = &p.language {
                            name_spans.push(Span::styled(
                                format!(" [{}]", lang),
                                Style::default()
                                    .fg(theme.border)
                                    .add_modifier(Modifier::ITALIC),
                            ));
                        }

                        let git_status = if let Some(branch) = &p.git_branch {
                            let mut spans =
                                vec![Span::styled("", Style::default().fg(theme.border))];
                            if p.has_changes {
                                spans[0] = Span::styled("", Style::default().fg(theme.git_dirty));
                            }
                            spans.push(Span::styled("  ", Style::default().fg(theme.no_git)));
                            spans.push(Span::styled(branch, Style::default().fg(theme.git_branch)));
                            Line::from(spans)
                        } else {
                            Line::from(vec![Span::styled(
                                " [no git]",
                                Style::default().fg(theme.no_git),
                            )])
                        };
                        let is_fav = app
                            .config
                            .favorites
                            .contains(&p.path.to_str().unwrap_or("").to_string());
                        let fav_cell = Cell::from(" ").style(Style::default().fg(if is_fav {
                            theme.git_dirty
                        } else {
                            theme.surface
                        }));
                        Row::new(vec![
                            Cell::from(Line::from(name_spans)),
                            Cell::from(git_status),
                            fav_cell,
                        ])
                    })
                    .collect()
            };
            let title = match app.mode {
                AppMode::Favorites => " Favorites ",
                AppMode::Recent => " Recently Opened ",
                _ => " Projects ",
            };
            let table = Table::new(
                rows,
                [
                    Constraint::Min(30),
                    Constraint::Length(30),
                    Constraint::Length(5),
                ],
            )
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            )
            .highlight_symbol(Span::styled("> ", Style::default().fg(theme.highlight)))
            .row_highlight_style(Style::default().bg(theme.surface));
            f.render_stateful_widget(table, chunks[1], &mut app.project_state);
        }
        AppMode::InputUrl => {
            let content = if app.input.is_empty() {
                Line::from(vec![Span::styled(
                    "Type or paste Git URL here...",
                    Style::default()
                        .fg(theme.no_git)
                        .add_modifier(Modifier::ITALIC),
                )])
            } else {
                Line::from(vec![Span::styled(
                    &app.input,
                    Style::default().fg(theme.git_dirty),
                )])
            };
            f.render_widget(
                Paragraph::new(content).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Git Repository URL ")
                        .border_style(Style::default().fg(theme.border)),
                ),
                chunks[1],
            );
        }
        AppMode::ChangeBaseDir => {
            let content = Line::from(vec![Span::styled(
                &app.input,
                Style::default().fg(theme.highlight),
            )]);
            f.render_widget(
                Paragraph::new(content).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" New Base Directory Path ")
                        .border_style(Style::default().fg(theme.border)),
                ),
                chunks[1],
            );
        }
    }

    if app.mode == AppMode::ConfirmOpen || app.mode == AppMode::Help {
        dim_background(f, &theme);
        let area = if app.mode == AppMode::Help {
            centered_rect(70, 70, f.area())
        } else {
            centered_rect(60, 20, f.area())
        };
        f.render_widget(Clear, area);
        if app.mode == AppMode::ConfirmOpen {
            if let Some(proj) = &app.pending_project {
                let block = Block::default()
                    .title(" Confirm ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.confirm_border));
                let text = format!(
                    "
Open {} in IntelliJ?

(y)es / (n)o",
                    proj.name
                );
                f.render_widget(
                    Paragraph::new(text)
                        .block(block)
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(theme.header_text)),
                    area,
                );
            }
        } else {
            let block = Block::default()
                .title(" Help & Shortcuts ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border));
            let help_rows = vec![
                Row::new(vec![Cell::from("hjkl / Arrows"), Cell::from("Navigate")]),
                Row::new(vec![
                    Cell::from("Enter / l"),
                    Cell::from("Select / Open / Confirm"),
                ]),
                Row::new(vec![
                    Cell::from("Backspace / h"),
                    Cell::from("Go Back / Cancel"),
                ]),
                Row::new(vec![Cell::from("/"), Cell::from("Search / Filter")]),
                Row::new(vec![Cell::from("f"), Cell::from("Toggle Favorite")]),
                Row::new(vec![Cell::from("t"), Cell::from("Open Quick Terminal")]),
                Row::new(vec![Cell::from("r"), Cell::from("Refresh Git Status")]),
                Row::new(vec![Cell::from("q"), Cell::from("Quit")]),
                Row::new(vec![
                    Cell::from("Esc"),
                    Cell::from("Clear Search / Main Menu"),
                ]),
                Row::new(vec![Cell::from("?"), Cell::from("Toggle Help")]),
            ];
            f.render_widget(
                Table::new(
                    help_rows,
                    [Constraint::Percentage(40), Constraint::Percentage(60)],
                )
                .block(block)
                .style(Style::default().fg(theme.header_text)),
                area,
            );
        }
    }

    let footer_text = if app.is_searching {
        format!("/{} (Press Enter to browse results)", app.search_query)
    } else if let Some((msg, _)) = &app.status_message {
        msg.clone()
    } else {
        match app.mode {
            AppMode::ConfirmOpen => "y: Yes  •  n: No / Cancel".to_string(),
            AppMode::Help => "Press any key to close".to_string(),
            AppMode::ThemeSelection => "Enter: Apply Theme  •  Backspace: Back".to_string(),
            AppMode::ChangeBaseDir => "Enter: Save Path  •  Backspace: Back".to_string(),
            AppMode::MainMenu => "Enter / Right: Select  •  ?: Help  •  q: Quit".to_string(),
            _ => "/: Search  •  r: Refresh  •  t: Terminal  •  f: Favorite  •  Backspace: Back  •  ?: Help".to_string(),
        }
    };
    f.render_widget(
        Paragraph::new(footer_text)
            .style(if app.status_message.is_some() {
                Style::default()
                    .fg(theme.git_clean)
                    .add_modifier(Modifier::BOLD)
            } else if app.is_searching {
                Style::default().fg(theme.git_dirty)
            } else {
                Style::default().fg(theme.header_text)
            })
            .alignment(Alignment::Center),
        chunks[2],
    );
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

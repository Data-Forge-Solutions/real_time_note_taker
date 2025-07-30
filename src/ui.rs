#![warn(clippy::pedantic)]
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use crate::{Action, App, Entry, InputMode, ThemeName};

fn key_to_string(key: KeyCode) -> String {
    match key {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        other => format!("{other:?}"),
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

/// Initializes the terminal for TUI rendering.
pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restores the terminal to its previous state.
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Runs the UI event loop with the provided application state.
pub fn run_ui(terminal: &mut Terminal<CrosstermBackend<Stdout>>, mut app: App) -> io::Result<App> {
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| draw(f, &app))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            let ev = event::read()?;
            if let CEvent::Key(key) = &ev {
                if key.kind != event::KeyEventKind::Press {
                    continue;
                }
                if key.code == app.keys.quit {
                    break;
                }
            }
            app.handle_event(&ev).ok();
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
    Ok(app)
}

#[allow(clippy::too_many_lines)]
fn draw(f: &mut ratatui::Frame<'_>, app: &App) {
    let theme = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(3),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(f.area());

    let notes: Vec<ListItem> = app
        .entries
        .iter()
        .map(|e| match e {
            Entry::Note(n) => ListItem::new(Line::from(vec![
                Span::styled(
                    n.timestamp.format("%H:%M:%S%.3f").to_string(),
                    Style::default()
                        .fg(theme.timestamp_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::styled(&n.text, Style::default().fg(theme.note_fg)),
            ])),
            Entry::Section(s) => ListItem::new(Line::from(vec![Span::styled(
                &s.title,
                Style::default()
                    .fg(theme.section_fg)
                    .add_modifier(Modifier::BOLD),
            )])),
        })
        .collect();

    let notes_list = List::new(notes)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(theme.notes_border))
                .title(Span::styled(
                    "Notes",
                    Style::default().fg(theme.notes_title),
                )),
        )
        .highlight_style(
            Style::default()
                .bg(theme.notes_highlight_bg)
                .fg(theme.notes_highlight_fg)
                .add_modifier(Modifier::BOLD),
        );

    let visible_height = usize::from(chunks[0].height.saturating_sub(2));
    let offset = app.entries.len().saturating_sub(visible_height);
    let mut state = ListState::default().with_offset(offset);
    if let Some(sel) = app.selected() {
        state.select(Some(sel));
    }
    f.render_stateful_widget(notes_list, chunks[0], &mut state);

    let input_title = match app.mode() {
        InputMode::EditingNote | InputMode::EditingExistingNote => {
            if let Some(time) = app.note_time() {
                format!("Note - {}", time.format("%H:%M:%S%.3f"))
            } else {
                "Note".to_string()
            }
        }
        InputMode::EditingSection | InputMode::EditingExistingSection => "Section".to_string(),
        InputMode::Saving => format!("Save File - {}", app.save_dir.display()),
        InputMode::Loading => format!("Load File - {}", app.save_dir.display()),
        InputMode::KeyBindings => "Key Bindings".to_string(),
        InputMode::ThemeSelect => "Select Theme".to_string(),
        InputMode::KeyCapture => "Set Key".to_string(),
        InputMode::ConfirmReplace => "Confirm".to_string(),
        InputMode::BindWarning => "Warning".to_string(),
        InputMode::Normal => "Input".to_string(),
    };

    let editing = matches!(
        app.mode(),
        InputMode::EditingNote
            | InputMode::EditingSection
            | InputMode::EditingExistingNote
            | InputMode::EditingExistingSection
            | InputMode::Saving
            | InputMode::Loading
    );

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(if editing {
            theme.editing_fg
        } else {
            theme.input_fg
        }))
        .title(Span::styled(
            input_title,
            Style::default().fg(theme.input_title),
        ));

    let input = Paragraph::new(app.input()).block(input_block);
    if matches!(
        app.mode(),
        InputMode::EditingNote
            | InputMode::EditingSection
            | InputMode::EditingExistingNote
            | InputMode::EditingExistingSection
            | InputMode::Saving
            | InputMode::Loading
    ) {
        let offset = u16::try_from(app.cursor()).unwrap_or(u16::MAX);
        f.set_cursor_position((
            chunks[1].x.saturating_add(offset.saturating_add(1)),
            chunks[1].y + 1,
        ));
    }
    f.render_widget(input, chunks[1]);

    let help_spans = vec![
        Span::styled(
            key_to_string(app.keys.new_note),
            Style::default().fg(theme.help_key),
        ),
        Span::styled(":New ", Style::default().fg(theme.help_desc)),
        Span::styled(
            key_to_string(app.keys.new_section),
            Style::default().fg(theme.help_key),
        ),
        Span::styled(":Section ", Style::default().fg(theme.help_desc)),
        Span::styled(
            key_to_string(app.keys.edit),
            Style::default().fg(theme.help_key),
        ),
        Span::styled(":Edit ", Style::default().fg(theme.help_desc)),
        Span::styled(
            key_to_string(app.keys.up),
            Style::default().fg(theme.help_key),
        ),
        Span::styled(":", Style::default().fg(theme.help_desc)),
        Span::styled(
            key_to_string(app.keys.down),
            Style::default().fg(theme.help_key),
        ),
        Span::styled(" ", Style::default().fg(theme.help_desc)),
        Span::styled(
            key_to_string(app.keys.save),
            Style::default().fg(theme.help_key),
        ),
        Span::styled(":Save ", Style::default().fg(theme.help_desc)),
        Span::styled(
            key_to_string(app.keys.load),
            Style::default().fg(theme.help_key),
        ),
        Span::styled(":Load ", Style::default().fg(theme.help_desc)),
        Span::styled(
            key_to_string(app.keys.bindings),
            Style::default().fg(theme.help_key),
        ),
        Span::styled(":Keys ", Style::default().fg(theme.help_desc)),
        Span::styled(
            key_to_string(app.keys.theme),
            Style::default().fg(theme.help_key),
        ),
        Span::styled(":Theme ", Style::default().fg(theme.help_desc)),
        Span::styled(
            key_to_string(app.keys.quit),
            Style::default().fg(theme.help_key),
        ),
        Span::styled(":Quit", Style::default().fg(theme.help_desc)),
    ];
    let help = Paragraph::new(Line::from(help_spans));
    f.render_widget(help, chunks[2]);

    if matches!(app.mode(), InputMode::Loading) {
        let area = centered_rect(60, 60, f.area());
        let items: Vec<ListItem> = if app.load_files.is_empty() {
            vec![ListItem::new("No files found")]
        } else {
            app.load_files
                .iter()
                .map(|p| ListItem::new(p.file_name().and_then(|n| n.to_str()).unwrap_or("")))
                .collect()
        };
        let mut state = ListState::default();
        if !app.load_files.is_empty() {
            state.select(Some(app.load_selected));
        }
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Select File",
                Style::default().fg(theme.overlay_title),
            ))
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(theme.overlay_border))
            .style(Style::default().bg(theme.overlay_bg));
        let list = List::new(items).block(block).highlight_style(
            Style::default()
                .bg(theme.overlay_highlight_bg)
                .fg(theme.overlay_highlight_fg)
                .add_modifier(Modifier::BOLD),
        );
        f.render_stateful_widget(list, area, &mut state);
    }

    if matches!(app.mode(), InputMode::KeyBindings) {
        let area = centered_rect(60, 60, f.area());
        let items: Vec<ListItem> = Action::ALL
            .iter()
            .map(|a| {
                let key = key_to_string(app.keys.get(*a));
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{a}: "), Style::default().fg(theme.help_desc)),
                    Span::styled(key, Style::default().fg(theme.help_key)),
                ]))
            })
            .collect();
        let mut state = ListState::default();
        state.select(Some(app.keybind_selected));
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Key Bindings",
                Style::default().fg(theme.overlay_title),
            ))
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(theme.overlay_border))
            .style(Style::default().bg(theme.overlay_bg));
        let list = List::new(items).block(block).highlight_style(
            Style::default()
                .bg(theme.overlay_highlight_bg)
                .fg(theme.overlay_highlight_fg)
                .add_modifier(Modifier::BOLD),
        );
        f.render_stateful_widget(list, area, &mut state);
    } else if matches!(app.mode(), InputMode::ThemeSelect) {
        let area = centered_rect(60, 60, f.area());
        let items: Vec<ListItem> = ThemeName::ALL
            .iter()
            .map(|t| {
                ListItem::new(Span::styled(
                    t.display_name(),
                    Style::default().fg(theme.overlay_text),
                ))
            })
            .collect();
        let mut state = ListState::default();
        state.select(Some(app.theme_selected));
        let block = Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Select Theme",
                Style::default().fg(theme.overlay_title),
            ))
            .border_type(BorderType::Plain)
            .border_style(Style::default().fg(theme.overlay_border))
            .style(Style::default().bg(theme.overlay_bg));
        let list = List::new(items).block(block).highlight_style(
            Style::default()
                .bg(theme.overlay_highlight_bg)
                .fg(theme.overlay_highlight_fg)
                .add_modifier(Modifier::BOLD),
        );
        f.render_stateful_widget(list, area, &mut state);
    } else if matches!(app.mode(), InputMode::KeyCapture) {
        if let Some(action) = app.capture_action {
            let area = centered_rect(60, 20, f.area());
            let msg = Paragraph::new(Line::from(vec![Span::raw(format!(
                "Press new key for {} (current: {})",
                action,
                key_to_string(app.keys.get(action))
            ))]))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.overlay_border))
                    .title(Span::styled(
                        "Set Key",
                        Style::default().fg(theme.overlay_title),
                    ))
                    .style(Style::default().bg(theme.overlay_bg)),
            );
            f.render_widget(msg, area);
        }
    } else if matches!(app.mode(), InputMode::ConfirmReplace) {
        if let (Some(key), Some(new_action), Some(conflict)) =
            (app.pending_key, app.pending_action, app.pending_conflict)
        {
            let area = centered_rect(60, 20, f.area());
            let msg = Paragraph::new(Line::from(vec![Span::raw(format!(
                "Bind {} to {} and unbind from {}?",
                key_to_string(key),
                new_action,
                conflict
            ))]))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.overlay_border))
                    .title(Span::styled(
                        "Confirm",
                        Style::default().fg(theme.overlay_title),
                    ))
                    .style(Style::default().bg(theme.overlay_bg)),
            );
            f.render_widget(msg, area);
        }
    } else if matches!(app.mode(), InputMode::BindWarning) {
        let area = centered_rect(60, 20, f.area());
        let msg = Paragraph::new(Line::from(vec![Span::styled(
            "Please choose a different key bind or rebind the Keys menu first.",
            Style::default().fg(theme.overlay_text),
        )]))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.overlay_border))
                .title(Span::styled(
                    "Warning",
                    Style::default().fg(theme.overlay_title),
                ))
                .style(Style::default().bg(theme.overlay_bg)),
        );
        f.render_widget(msg, area);
    }
}

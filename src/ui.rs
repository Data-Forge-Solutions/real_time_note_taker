#![warn(clippy::pedantic)]
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;
use std::io::{self, Stdout};
use std::time::{Duration, Instant};

use crate::{App, Entry, InputMode};

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
pub fn run_ui(terminal: &mut Terminal<CrosstermBackend<Stdout>>, mut app: App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| draw(f, &app))?;
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if crossterm::event::poll(timeout)? {
            match event::read()? {
                CEvent::Key(key) if key.code == KeyCode::Char('q') => break,
                ev => {
                    app.handle_event(ev).ok();
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
    Ok(())
}

fn draw(f: &mut ratatui::Frame<'_>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(f.area());

    let notes: Vec<ListItem> = app
        .entries
        .iter()
        .map(|e| match e {
            Entry::Note(n) => ListItem::new(Line::from(vec![
                Span::styled(
                    n.timestamp.format("%H:%M:%S%.3f").to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" - "),
                Span::raw(&n.text),
            ])),
            Entry::Section(s) => ListItem::new(Line::from(vec![Span::styled(
                &s.title,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )])),
        })
        .collect();

    let notes_list = List::new(notes).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .title("Notes"),
    );
    f.render_widget(notes_list, chunks[0]);

    let input_title = match app.mode() {
        InputMode::EditingNote => {
            if let Some(time) = app.note_time() {
                format!("Note - {}", time.format("%H:%M:%S%.3f"))
            } else {
                "Note".to_string()
            }
        }
        InputMode::EditingSection => "Section".to_string(),
        InputMode::Normal => "Input".to_string(),
    };

    let mut input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .title(input_title);
    if matches!(
        app.mode(),
        InputMode::EditingNote | InputMode::EditingSection
    ) {
        input_block = input_block.style(Style::default().fg(Color::Yellow));
    }

    let input = Paragraph::new(app.input()).block(input_block);
    if matches!(
        app.mode(),
        InputMode::EditingNote | InputMode::EditingSection
    ) {
        let offset = u16::try_from(app.input().len()).unwrap_or(u16::MAX);
        f.set_cursor_position((
            chunks[1].x.saturating_add(offset.saturating_add(1)),
            chunks[1].y + 1,
        ));
    }
    f.render_widget(input, chunks[1]);
}

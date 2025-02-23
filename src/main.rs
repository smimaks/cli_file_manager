use std::io::Stdout;
use std::{io, thread, time::Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Widget},
    Frame, Terminal,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::text::Spans;
use tui::widgets::Paragraph;

mod app;
use app::FileManager;

fn main() -> io::Result<()> {
    // init terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // init App
    let mut file_manager = FileManager::new().expect("Error FileManager init");

    loop {
        terminal.draw(|f| ui(f, &file_manager))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Down => file_manager.down(),
                KeyCode::Up => file_manager.up(),
                KeyCode::PageDown => file_manager.page_down(),
                KeyCode::PageUp => file_manager.page_up(),
                KeyCode::Enter | KeyCode::Right => file_manager.enter_handler()?,
                KeyCode::Backspace | KeyCode::Left => file_manager.to_parent_dir()?,
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, file_manager: &FileManager) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(f.size());

    let mut state = ListState::default();

    state.select(Some(file_manager.selected));
    let items: Vec<ListItem> = file_manager
        .files
        .iter()
        .enumerate()
        .map(|(i, path)| {
            let style = if i == file_manager.selected {
                Style::default().fg(Color::Blue)
            } else {
                Style::default()
            };
            let display_name = if path.is_dir() {
                format!("📁 {}", path.file_name().unwrap().to_string_lossy())
            } else {
                format!("📄 {}", path.file_name().unwrap().to_string_lossy())
            };
            ListItem::new(Span::styled(display_name, style))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Files"))
        .highlight_style(Style::default().bg(Color::Yellow))
        .highlight_symbol(">> ");

    f.render_stateful_widget(list, chunks[0], &mut state);

    if let Some(content) = &file_manager.content {
        let paragraph = Paragraph::new(content.as_ref())
            .block(Block::default().borders(Borders::ALL).title("File Content"))
            .scroll((file_manager.file_scroll as u16, 0));
        f.render_widget(paragraph, chunks[1]);
    } else {
        let paragraph = Paragraph::new("No file selected")
            .block(Block::default().borders(Borders::ALL).title("File Content"));
        f.render_widget(paragraph, chunks[1]);
    }
}

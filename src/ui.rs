use crate::app::{FileManager, InputMode, Mode};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::Frame;

pub fn ui<B: Backend>(f: &mut Frame<B>, file_manager: &FileManager) {
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
                Style::default().fg(Color::White)
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
        .highlight_symbol("|-> ");

    f.render_stateful_widget(list, chunks[0], &mut state);

    match file_manager.mode {
        Mode::Normal => {
            if let Some(content) = &file_manager.content {
                let paragraph = Paragraph::new(content.as_ref())
                    .block(Block::default().borders(Borders::ALL).title("File"))
                    .scroll((file_manager.file_scroll as u16, 0));
                f.render_widget(paragraph, chunks[1]);
            } else {
                let paragraph = Paragraph::new("No file selected")
                    .block(Block::default().borders(Borders::ALL).title("File Content"));
                f.render_widget(paragraph, chunks[1]);
            }
        }
        Mode::Menu => {
          match file_manager.input_mode {
              InputMode::Input => {
                  let input = Paragraph::new(file_manager.input_buffer.as_ref())
                      .block(Block::default().borders(Borders::ALL).title("Введите имя: "));
                  f.render_widget(input, chunks[1]);
              }
              InputMode::Normal => {
                  let menu_items = file_manager.show_menu();
                  let items: Vec<ListItem> = menu_items
                      .iter()
                      .enumerate()
                      .map(|(i, action)| {
                          let style = if i == file_manager.menu_selected {
                              Style::default().fg(Color::LightBlue)
                          } else {
                              Style::default()
                          };
                          ListItem::new(Span::styled(*action, style))
                      })
                      .collect();
                  let menu = List::new(items)
                      .block(Block::default().borders(Borders::ALL).title("Menu"))
                      .highlight_symbol("|-> ");
                  f.render_widget(menu, chunks[1]);
              }
          }
        }
    }
}

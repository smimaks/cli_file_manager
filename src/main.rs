use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    widgets::Widget,
    Terminal,
};

use crossterm::{
    event::{self, Event, KeyCode, ModifierKeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};
use crossterm::event::{KeyEvent, KeyModifiers};

mod app;
mod ui;
use app::FileManager;
use ui::ui;
use crate::app::{InputMode, Mode};

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

        match file_manager.get_mode() {
            Mode::Normal => {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('m') => file_manager.enable_menu_mode(),
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
            Mode::Menu => {
               match file_manager.get_input_mode() {
                   InputMode::Input => {
                       if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                           if modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('c') {
                               file_manager.disable_input_mode()
                           }
                           match code {
                               KeyCode::Char(c) => file_manager.add_to_input_buffer(c),
                               KeyCode::Backspace => file_manager.delete_from_input_buffer(),
                               KeyCode::Enter => {
                                   file_manager.handle_menu_action()?;
                               }
                               _ => {}
                           }
                       }

                   }
                   InputMode::Normal => {
                       if let Event::Key(key) = event::read()? {
                           match key.code {
                               KeyCode::Esc | KeyCode::Backspace => file_manager.disable_menu_mode(),
                               KeyCode::Down => file_manager.menu_down(),
                               KeyCode::Up => file_manager.menu_up(),
                               KeyCode::Enter | KeyCode::Right => file_manager.select_from_menu()?,
                               _ => {}
                           }
                       }
                   }
               }
            }
        }

    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

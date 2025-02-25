use std::io;
use tui::{
    backend::{Backend, CrosstermBackend},
    widgets::Widget,
    Terminal,
};

use crossterm::{
    event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen},
};

mod app;
mod input_handler;
mod ui;
use crate::app::{FileManager, Mode};
use ui::render;

fn main() -> io::Result<()> {
    // init terminal
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // init App
    let mut file_manager = FileManager::new().expect("Error FileManager init");

    loop {
        terminal.draw(|f| render(f, &file_manager))?;

        let event = event::read()?;
        match file_manager.get_mode() {
            Mode::Normal => input_handler::handle_normal_mode(event, &mut file_manager)?,
            Mode::Menu => input_handler::handle_menu_mode(event, &mut file_manager)?,
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

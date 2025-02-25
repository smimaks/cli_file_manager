use crate::app::{FileManager, InputMode};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::io;

pub fn handle_normal_mode(event: Event, file_manager: &mut FileManager) -> io::Result<()> {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Char('q') => return Err(io::Error::new(io::ErrorKind::Interrupted, "Quit")),
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
    Ok(())
}

pub fn handle_menu_mode(event: Event, file_manager: &mut FileManager) -> io::Result<()> {
    match file_manager.get_input_mode() {
        InputMode::Input => {
            handle_input_mode(event, file_manager).unwrap_or_else(|err| eprintln!("Error: {}", err))
        }
        InputMode::Normal => handle_normal_input_mode(event, file_manager)
            .unwrap_or_else(|err| eprintln!("Error: {}", err)),
    }

    Ok(())
}

fn handle_input_mode(event: Event, file_manager: &mut FileManager) -> io::Result<()> {
    if let Event::Key(KeyEvent {
        code, modifiers, ..
    }) = event
    {
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
    Ok(())
}

fn handle_normal_input_mode(event: Event, file_manager: &mut FileManager) -> io::Result<()> {
    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace => file_manager.disable_menu_mode(),
            KeyCode::Down => file_manager.menu_down(),
            KeyCode::Up => file_manager.menu_up(),
            KeyCode::Enter | KeyCode::Right => file_manager.select_from_menu()?,
            _ => {}
        }
    }
    Ok(())
}

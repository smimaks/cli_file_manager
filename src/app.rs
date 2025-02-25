use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::{
    fs::{self, File},
    io::{self, BufRead},
};

pub enum Mode {
    Normal,
    Menu,
}

pub enum InputMode {
    Input,
    Normal,
}

pub enum MenuAction {
    CreateFile,
    CreateDir,
    Rename,
    Delete,
    Cancel,
}

pub struct FileManager {
    current_dir: PathBuf,
    files: Vec<PathBuf>,
    selected: usize,
    content: Option<String>,
    scroll: usize,
    file_scroll: usize,
    file_lines_count: usize,
    mode: Mode,
    input_mode: InputMode,
    input_buffer: String,
    menu_action: Option<MenuAction>,
    menu_selected: usize,
}

impl FileManager {
    pub fn new() -> io::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let files = Self::files_list(&current_dir)?;

        Ok(Self {
            current_dir,
            files,
            selected: 0,
            content: None,
            scroll: 0,
            file_scroll: 0,
            file_lines_count: 0,
            mode: Mode::Normal,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            menu_action: None,
            menu_selected: 0,
        })
    }

    // Getters
    pub fn get_mode(&self) -> &Mode {
        &self.mode
    }

    pub fn get_input_mode(&self) -> &InputMode {
        &self.input_mode
    }

    pub fn get_selected(&self) -> &usize {
        &self.selected
    }

    pub fn get_files(&self) -> &Vec<PathBuf> {
        &self.files
    }

    pub fn get_content(&self) -> &Option<String> {
        &self.content
    }

    pub fn get_file_scroll(&self) -> &usize {
        &self.file_scroll
    }

    pub fn get_input_buffer(&self) -> &String {
        &self.input_buffer
    }

    pub fn get_menu_selected(&self) -> &usize {
        &self.menu_selected
    }

    // Setters
    pub fn add_to_input_buffer(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn delete_from_input_buffer(&mut self) {
        self.input_buffer.pop();
    }

    pub fn files_list(path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut files = fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        files.sort();
        Ok(files)
    }

    // Navigation
    pub fn enter_handler(&mut self) -> io::Result<()> {
        if let Some(path) = self.files.get(self.selected) {
            if path.is_dir() {
                self.enter_dir()?;
            } else {
                self.open_file()?;
            }
        }
        Ok(())
    }

    fn enter_dir(&mut self) -> io::Result<()> {
        if let Some(path) = self.files.get(self.selected) {
            self.current_dir = path.to_path_buf();
            self.files = Self::files_list(&self.current_dir)?;
            self.selected = 0;
        }
        Ok(())
    }

    fn open_file(&mut self) -> io::Result<()> {
        if let Some(path) = self.files.get(self.selected) {
            if path.is_file() {
                self.content = Some(fs::read_to_string(path)?);
                self.file_lines_count = Self::get_file_lines_count(path);
            } else {
                self.content = None
            }
        }
        Ok(())
    }

    pub fn to_parent_dir(&mut self) -> io::Result<()> {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.files = Self::files_list(&self.current_dir)?;
            self.selected = 0;
            self.content = None;
        }
        Ok(())
    }

    pub fn up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else {
            self.selected = self.files.len() - 1;
        }
        self.file_scroll = 0;
        self.open_file().unwrap();
    }

    pub fn menu_up(&mut self) {
        if self.menu_selected > 0 {
            self.menu_selected -= 1
        } else {
            self.menu_selected = self.show_menu().len() - 1;
        }
    }

    pub fn page_up(&mut self) {
        if self.file_scroll >= 2 {
            self.file_scroll -= 2;
        }
    }

    pub fn down(&mut self) {
        if self.selected < self.files.len() - 1 {
            self.selected += 1;
            self.file_scroll = 0;
            self.open_file().unwrap();
        } else {
            self.selected = 0
        }

        self.file_scroll = 0;
        self.open_file().unwrap();
    }

    pub fn menu_down(&mut self) {
        if self.menu_selected < self.show_menu().len() - 1 {
            self.menu_selected += 1;
        } else {
            self.menu_selected = 0;
        }
    }

    pub fn page_down(&mut self) {
        if self.file_scroll <= self.file_lines_count - 1 {
            self.file_scroll += 2;
        }
    }

    fn get_file_lines_count(path: &PathBuf) -> usize {
        let file = File::open(path).expect("Open file error");
        let reader = BufReader::new(file);
        reader.lines().count()
    }

    //  Modes
    pub fn enable_menu_mode(&mut self) {
        self.mode = Mode::Menu;
    }

    pub fn disable_menu_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn enable_input_mode(&mut self) {
        self.input_mode = InputMode::Input
    }

    pub fn disable_input_mode(&mut self) {
        self.input_mode = InputMode::Normal
    }

    // Menu

    pub fn show_menu(&self) -> Vec<&str> {
        vec![
            "Удалить",
            "Создать файл",
            "Создать папку",
            "Переименовать",
            "Отмена",
        ]
    }

    pub fn select_from_menu(&mut self) -> io::Result<()> {
        match self.menu_selected {
            0 => self.delete_selected()?,
            1 => {
                self.input_mode = InputMode::Input;
                self.menu_action = Option::from(MenuAction::CreateFile);
            }
            2 => {
                self.input_mode = InputMode::Input;
                self.menu_action = Option::from(MenuAction::CreateDir);
            }
            3 => {
                self.input_mode = InputMode::Input;
                self.menu_action = Option::from(MenuAction::Rename);
            }
            _ => {
                self.mode = Mode::Normal;
            }
        }
        Ok(())
    }

    pub fn handle_menu_action(&mut self) -> io::Result<()> {
        if let Some(action) = self.menu_action.take() {
            match action {
                MenuAction::CreateFile => self.create_file()?,
                MenuAction::CreateDir => self.create_dir()?,
                MenuAction::Rename => self.rename_selected()?,
                _ => {}
            }
        };
        self.disable_input_mode();
        Ok(())
    }

    fn update_file_list(&mut self) -> io::Result<()> {
        let mut files = fs::read_dir(&self.current_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        files.sort();

        self.files = files;
        Ok(())
    }

    fn create_file(&mut self) -> io::Result<()> {
        let file_name = self.input_buffer.trim();
        if !file_name.is_empty() {
            let file_path = self.current_dir.join(file_name);
            File::create(file_path)?;
            self.update_file_list()?;
        }
        self.input_buffer.clear();
        Ok(())
    }

    fn create_dir(&mut self) -> io::Result<()> {
        let dir_name = self.input_buffer.trim();
        if !dir_name.is_empty() {
            let dir_path = self.current_dir.join(dir_name);
            fs::create_dir(dir_path)?;
            self.update_file_list()?;
        }
        self.input_buffer.clear();
        Ok(())
    }

    fn rename_selected(&mut self) -> io::Result<()> {
        let new_name = self.input_buffer.trim();
        if let Some(path) = self.files.get(self.selected) {
            if !new_name.is_empty() {
                let new_path = path.parent().unwrap().join(new_name);
                fs::rename(path, new_path)?;
                self.update_file_list()?;
            }
        }
        self.input_buffer.clear();
        self.update_file_list()?;
        Ok(())
    }

    fn delete_selected(&mut self) -> io::Result<()> {
        if let Some(path) = self.files.get(self.selected) {
            if path.is_dir() {
                fs::remove_dir_all(path)?;
            } else {
                fs::remove_file(path)?;
            }
        };
        self.update_file_list()?;
        Ok(())
    }
}

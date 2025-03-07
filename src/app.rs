use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::{
    fs::{self, File},
    io::{self, BufRead},
};

pub enum Mode {
    Normal,
    Menu,
    Search,
    Context,
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

pub enum ContextAction {
    Open,
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
    context_action: Option<ContextAction>,
    context_selected: usize,
    search_buffer: String,
}

impl FileManager {
    pub fn new() -> io::Result<Self> {
        let current_dir = std::env::current_dir()?;
        let files = Self::get_file_list(&current_dir)?;

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
            context_action: None,
            context_selected: 0,
            search_buffer: String::new(),
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

    pub fn get_search_buffer(&self) -> &String {
        &self.search_buffer
    }

    pub fn get_menu_selected(&self) -> &usize {
        &self.menu_selected
    }

    pub fn get_context_selected(&self) -> &usize {
        &self.context_selected
    }

    pub fn get_file_list(path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut files = fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        files.sort();
        Ok(files)
    }

    pub fn add_to_input_buffer(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn delete_from_input_buffer(&mut self) {
        self.input_buffer.pop();
    }

    pub fn add_to_search_buffer(&mut self, c: char) {
        self.search_buffer.push(c);
    }

    pub fn delete_from_search_buffer(&mut self) {
        self.search_buffer.pop();
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
            self.files = Self::get_file_list(&self.current_dir)?;
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
            self.files = Self::get_file_list(&self.current_dir)?;
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

    pub fn context_up(&mut self) {
        if self.context_selected > 0 {
            self.context_selected -= 1
        } else {
            self.context_selected = self.show_menu().len() - 1;
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

    pub fn context_down(&mut self) {
        if self.context_selected < self.show_context().len() - 1 {
            self.context_selected += 1;
        } else {
            self.context_selected = 0;
        }
    }

    pub fn page_down(&mut self) {
        if self.file_scroll <= self.file_lines_count - 1 {
            self.file_scroll += 2;
        }
    }

    pub fn find_in_current_dir(&mut self) {
        let file_name = self.search_buffer.trim();
        if file_name.is_empty() {
            return;
        }

        let matcher = SkimMatcherV2::default();

        if let Some((index, _)) = self.files.iter().enumerate().find(|(_, path)| {
            path.file_name() // Получаем имя файла
                .and_then(|os_str| os_str.to_str())
                .map_or(false, |name| matcher.fuzzy_match(name, file_name).is_some())
        }) {
            self.selected = index;
        } else {
            println!("No file matches '{}'.", file_name);
        }

        self.search_buffer.clear();
        self.default_input_mode();
        self.default_mode();
    }

    fn get_file_lines_count(path: &PathBuf) -> usize {
        let file = File::open(path).expect("Open file error");
        let reader = BufReader::new(file);
        reader.lines().count()
    }

    //  Modes
    pub fn menu_mode(&mut self) {
        self.mode = Mode::Menu;
    }

    pub fn default_mode(&mut self) {
        self.mode = Mode::Normal;
    }

    pub fn search_mode(&mut self) {
        self.mode = Mode::Search;
    }

    pub fn context_mode(&mut self) {
        self.mode = Mode::Context
    }

    pub fn input_mode(&mut self) {
        self.input_mode = InputMode::Input
    }

    pub fn default_input_mode(&mut self) {
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

    pub fn show_context(&self) -> Vec<&str> {
        vec![
            "Открыть в Nano",
            "Открыть в Vim",
            "Открыть в WebStorm",
            "Открыть в RustRover",
            "Открыть в VS Code",
            "Отмена",
        ]
    }

    pub fn select_from_menu(&mut self) -> io::Result<()> {
        match self.menu_selected {
            0 => self.delete_selected()?,
            1 => {
                self.input_mode();
                self.menu_action = Option::from(MenuAction::CreateFile);
            }
            2 => {
                self.input_mode();
                self.menu_action = Option::from(MenuAction::CreateDir);
            }
            3 => {
                self.input_mode();
                self.menu_action = Option::from(MenuAction::Rename);
            }
            _ => self.default_input_mode(),
        }
        Ok(())
    }

    pub fn select_from_context(&mut self) -> io::Result<()> {
        let file_path = self.files.get(self.selected);
        let editors = vec!["nano", "vim", "webstorm", "rustrover", "code"];
        let current_editor = editors.get(self.context_selected);

        if let Some(editor) = current_editor {
            if let Some(file) = file_path {
                Self::opn_in_editor(file, editor)?;
            }
        } else {
            self.default_mode()
        }

        // match self.context_selected {
        //     0 => {
        //         if let Some(path) = file_path  {
        //             Self::opn_in_editor(path, current_editor)?;
        //         }
        //     }
        //     1 => println!("Open in Vim"),
        //     2 => println!("Open in WS"),
        //     3 => println!("Open in RustRover"),
        //     4 => println!("Open in VS CODE"),
        //     _ => self.default_mode(),
        // }
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
        self.default_input_mode();
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

    // open files in editor

    fn opn_in_editor(file_path: &PathBuf, editor: &str) -> Result<(), io::Error> {
        Command::new(editor)
            .arg(file_path)
            .stdout(Stdio::null()) // Перенаправляем stdout в /dev/null
            .stderr(Stdio::null()) // Перенаправляем stderr в /dev/null
            .spawn()?
            .wait()?;
        Ok(())
    }
}

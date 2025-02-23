use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::{
    fs::{self, File},
    io::{self, BufRead},
};

pub struct FileManager {
    current_dir: PathBuf,
    pub(crate) files: Vec<PathBuf>,
    pub(crate) selected: usize,
    pub(crate) content: Option<String>,
    pub(crate) scroll: usize,
    pub(crate) file_scroll: usize,
    file_lines_count: usize,
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
        })
    }

    pub fn files_list(path: &Path) -> io::Result<Vec<PathBuf>> {
        let mut files = fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        files.sort();
        Ok(files)
    }

    pub fn enter_handler(&mut self) -> io::Result<()> {
        if let Some(path) = self.files.get(self.selected) {
            if path.is_dir() {
                Self::enter_dir(self, self.selected)?;
            } else {
                Self::open_file(self, self.selected)?;
            }
        }
        Ok(())
    }

    fn enter_dir(&mut self, index: usize) -> io::Result<()> {
        if let Some(path) = self.files.get(index) {
            self.current_dir = path.to_path_buf();
            self.files = Self::files_list(&self.current_dir)?;
            self.selected = 0;
        }
        Ok(())
    }

    fn open_file(&mut self, index: usize) -> io::Result<()> {
        if let Some(path) = self.files.get(index) {
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
        }
    }

    pub fn page_up(&mut self) {
        if self.file_scroll >= 2 {
            self.file_scroll -= 2;
        }
    }

    pub fn page_down(&mut self) {
        if self.file_scroll <= self.file_lines_count - 1 {
            self.file_scroll += 2;
        }

    }

    pub fn down(&mut self) {
        if self.selected < self.files.len() - 1 {
            self.selected += 1;
        }
    }

    fn get_file_lines_count(path: &PathBuf) -> usize {
        let file = File::open(path).expect("Open file error");
        let reader = BufReader::new(file);
        reader.lines().count()
    }
}

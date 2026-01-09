use anyhow::Result;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
}

pub struct FileBrowser {
    pub current_dir: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl FileBrowser {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut browser = Self {
            current_dir: path,
            entries: Vec::new(),
            selected: 0,
            scroll_offset: 0,
        };
        browser.refresh()?;
        Ok(browser)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.entries.clear();

        let mut entries: Vec<FileEntry> = fs::read_dir(&self.current_dir)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip hidden files
                if name.starts_with('.') {
                    return None;
                }

                let is_dir = path.is_dir();

                // Only show directories and audio files
                if is_dir {
                    Some(FileEntry { path, name, is_dir })
                } else if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ["mp3", "flac", "m4a", "mp4", "aac", "ogg"].contains(&ext_str.as_str()) {
                        Some(FileEntry { path, name, is_dir })
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        // Sort: directories first, then files, both alphabetically
        entries.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        self.entries = entries;

        // Reset selection if out of bounds
        if self.selected >= self.entries.len() && !self.entries.is_empty() {
            self.selected = self.entries.len() - 1;
        }

        Ok(())
    }

    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn next(&mut self) {
        if !self.entries.is_empty() && self.selected < self.entries.len() - 1 {
            self.selected += 1;
        }
    }

    pub fn get_selected_path(&self) -> Option<PathBuf> {
        self.entries.get(self.selected).map(|e| e.path.clone())
    }

    pub fn enter_directory(&mut self, path: PathBuf) -> Result<()> {
        self.current_dir = path;
        self.selected = 0;
        self.scroll_offset = 0;
        self.refresh()?;
        Ok(())
    }

    pub fn go_up(&mut self) -> Result<()> {
        if let Some(parent) = self.current_dir.parent() {
            self.current_dir = parent.to_path_buf();
            self.selected = 0;
            self.scroll_offset = 0;
            self.refresh()?;
        }
        Ok(())
    }
}

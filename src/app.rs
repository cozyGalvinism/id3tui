use anyhow::Result;
use std::path::PathBuf;

use crate::file_browser::FileBrowser;
use crate::metadata::AudioMetadata;

#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

#[derive(Debug, PartialEq)]
pub enum Focus {
    FileBrowser,
    MetadataView,
}

pub struct App {
    pub input_mode: InputMode,
    pub focus: Focus,
    pub file_browser: FileBrowser,
    pub current_file: Option<PathBuf>,
    pub metadata: Option<AudioMetadata>,
    pub show_help: bool,
    pub editing_field: usize,
    pub cursor_position: usize,
    pub scroll_offset: usize,
}

impl App {
    pub fn new(path: Option<PathBuf>) -> Result<Self> {
        let start_dir = if let Some(p) = path {
            if p.is_absolute() {
                p
            } else {
                std::env::current_dir()?.join(p)
            }
        } else {
            std::env::current_dir()?
        };

        // Canonicalize the path to resolve any . or .. components
        let start_dir = start_dir.canonicalize().unwrap_or(start_dir);

        Ok(Self {
            input_mode: InputMode::Normal,
            focus: Focus::FileBrowser,
            file_browser: FileBrowser::new(start_dir)?,
            current_file: None,
            metadata: None,
            show_help: false,
            editing_field: 0,
            cursor_position: 0,
            scroll_offset: 0,
        })
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_focus(&mut self) {
        if self.current_file.is_some() {
            self.focus = match self.focus {
                Focus::FileBrowser => Focus::MetadataView,
                Focus::MetadataView => Focus::FileBrowser,
            };
        }
    }

    pub fn enter_edit_mode(&mut self) {
        if self.focus == Focus::MetadataView && self.metadata.is_some() {
            self.input_mode = InputMode::Editing;
            self.cursor_position = self.metadata.as_ref().unwrap().get_field_value(self.editing_field).chars().count();
            self.scroll_offset = 0;
        }
    }

    pub fn exit_edit_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.focus = Focus::FileBrowser;
    }

    pub fn previous_item(&mut self) {
        match self.focus {
            Focus::FileBrowser => self.file_browser.previous(),
            Focus::MetadataView => {
                if self.metadata.is_some() {
                    // Do nothing in normal mode, handled by previous_field in edit mode
                }
            }
        }
    }

    pub fn next_item(&mut self) {
        match self.focus {
            Focus::FileBrowser => self.file_browser.next(),
            Focus::MetadataView => {
                if self.metadata.is_some() {
                    // Do nothing in normal mode, handled by next_field in edit mode
                }
            }
        }
    }

    pub fn select_item(&mut self) -> Result<()> {
        if self.focus == Focus::FileBrowser {
            if let Some(path) = self.file_browser.get_selected_path() {
                if path.is_dir() {
                    self.file_browser.enter_directory(path)?;
                } else if path.is_file() {
                    // Check if it's an audio file
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if ["mp3", "flac", "m4a", "mp4", "aac", "ogg"].contains(&ext_str.as_str()) {
                            self.current_file = Some(path.clone());
                            self.metadata = Some(AudioMetadata::read_from_file(&path)?);
                            self.focus = Focus::MetadataView;
                            self.editing_field = 0;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn go_back(&mut self) -> Result<()> {
        if self.focus == Focus::FileBrowser {
            self.file_browser.go_up()?;
        }
        Ok(())
    }

    pub fn previous_field(&mut self) {
        if self.metadata.is_some() {
            if self.editing_field > 0 {
                self.editing_field -= 1;
                self.cursor_position = self.metadata.as_ref().unwrap().get_field_value(self.editing_field).chars().count();
                self.scroll_offset = 0;
            }
        }
    }

    pub fn next_field(&mut self) {
        if let Some(metadata) = &self.metadata {
            if self.editing_field < metadata.field_count() - 1 {
                self.editing_field += 1;
                self.cursor_position = self.metadata.as_ref().unwrap().get_field_value(self.editing_field).chars().count();
                self.scroll_offset = 0;
            }
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if let Some(metadata) = &mut self.metadata {
            let value = metadata.get_field_value(self.editing_field);
            let mut chars: Vec<char> = value.chars().collect();
            chars.insert(self.cursor_position, c);
            let new_value: String = chars.into_iter().collect();
            metadata.set_field_value(self.editing_field, new_value);
            self.cursor_position += 1;
        }
    }

    pub fn delete_char(&mut self) {
        if let Some(metadata) = &mut self.metadata {
            if self.cursor_position > 0 {
                let value = metadata.get_field_value(self.editing_field);
                let mut chars: Vec<char> = value.chars().collect();
                chars.remove(self.cursor_position - 1);
                let new_value: String = chars.into_iter().collect();
                metadata.set_field_value(self.editing_field, new_value);
                self.cursor_position -= 1;
            }
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        if let Some(metadata) = &self.metadata {
            let value = metadata.get_field_value(self.editing_field);
            let char_count = value.chars().count();
            if self.cursor_position < char_count {
                self.cursor_position += 1;
            }
        }
    }

    pub fn save_field(&mut self) -> Result<()> {
        if let (Some(metadata), Some(path)) = (&self.metadata, &self.current_file) {
            metadata.write_to_file(path)?;
            self.input_mode = InputMode::Normal;
            self.focus = Focus::FileBrowser;
        }
        Ok(())
    }

    pub fn update_scroll(&mut self, viewport_width: usize) {
        // Account for the label width and formatting (e.g., "Title       : ")
        let label_width = 14; // "Title       : " is 14 chars
        let usable_width = if viewport_width > label_width {
            viewport_width.saturating_sub(label_width)
        } else {
            1
        };

        // Ensure cursor is visible within the viewport
        // The cursor character "█" is rendered at the cursor position
        if self.cursor_position < self.scroll_offset {
            // Cursor moved before the visible area, scroll left
            self.scroll_offset = self.cursor_position;
        } else if self.cursor_position >= self.scroll_offset + usable_width {
            // Cursor moved beyond the visible area, scroll right
            // We need to ensure cursor_position - scroll_offset < usable_width
            // So: scroll_offset = cursor_position - (usable_width - 1)
            self.scroll_offset = self.cursor_position.saturating_sub(usable_width - 1);
        }
    }
}

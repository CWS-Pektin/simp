use std::path::PathBuf;
use std::time::SystemTime;

use iced::widget::{markdown, text_editor};

#[derive(Debug)]
pub struct OpenDocument {
    pub path: PathBuf,
    pub text: String,
    pub editor: text_editor::Content,
    pub markdown_items: Vec<markdown::Item>,
    pub dirty: bool,
    pub save_pending: bool,
    pub last_saved_text: Option<String>,
    pub last_disk_mtime: Option<SystemTime>,
}

impl OpenDocument {
    pub fn new(path: PathBuf, text: String, mtime: Option<SystemTime>) -> Self {
        let editor = text_editor::Content::with_text(&text);
        let markdown_items = markdown::parse(&text).collect();
        Self {
            path,
            text: text.clone(),
            editor,
            markdown_items,
            dirty: false,
            save_pending: false,
            last_saved_text: Some(text),
            last_disk_mtime: mtime,
        }
    }

    pub fn sync_text_from_editor(&mut self) {
        self.text = self.editor.text();
        self.markdown_items = markdown::parse(&self.text).collect();
    }

    pub fn replace_content(&mut self, text: String, mtime: Option<SystemTime>) {
        self.text = text.clone();
        self.editor = text_editor::Content::with_text(&text);
        self.markdown_items = markdown::parse(&text).collect();
        self.dirty = false;
        self.save_pending = false;
        self.last_saved_text = Some(text);
        self.last_disk_mtime = mtime;
    }
}

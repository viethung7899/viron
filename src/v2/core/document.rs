use crate::core::history::change::Change;
use crate::core::language::Language;
use crate::core::{buffer::Buffer, history::History};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Document {
    pub buffer: Buffer,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub language: Language,
    pub history: History,
}

impl Document {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::default(),
            path: None,
            modified: false,
            language: Language::PlainText,
            history: History::new(1000),
        }
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path.display()))?;

        let language = Language::from_path(path);

        Ok(Self {
            buffer: Buffer::from_string(&content),
            path: Some(path.to_path_buf()),
            modified: false,
            language,
            history: History::new(1000),
        })
    }

    pub fn save(&mut self) -> Result<()> {
        if let Some(path) = &self.path {
            let content = self.buffer.to_bytes();
            std::fs::write(path, content)
                .context(format!("Failed to write to file: {}", path.display()))?;
            self.modified = false;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No file path set"))
        }
    }

    pub fn save_as(&mut self, path: &Path) -> Result<()> {
        self.path = Some(path.to_path_buf());
        self.save()
    }

    pub fn mark_modified(&mut self) {
        self.modified = true;
    }

    pub fn file_name(&self) -> Option<String> {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
    }

    pub fn full_file_path(&self) -> Option<PathBuf> {
        let current = std::env::current_dir().ok()?;
        self.path
            .as_ref()
            .map(|p| current.join(p))
    }

    pub fn undo(&mut self) -> Result<Change> {
        if let Some(change) = self.history.undo() {
            self.mark_modified();
            self.buffer.apply_change(&change);
            Ok(change)
        } else {
            Err(anyhow::anyhow!("No changes to undo"))
        }
    }

    pub fn redo(&mut self) -> Result<Change> {
        if let Some(change) = self.history.redo() {
            self.mark_modified();
            self.buffer.apply_change(&change);
            Ok(change)
        } else {
            Err(anyhow::anyhow!("No changes to redo"))
        }
    }
}

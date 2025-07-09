use crate::core::history::edit::Edit;
use crate::core::language::Language;
use crate::core::syntax::SyntaxEngine;
use crate::core::{buffer::Buffer, history::History};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct Document {
    pub buffer: Buffer,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub language: Language,
    pub syntax_engine: Option<SyntaxEngine>,
    pub version: usize,
    pub history: History,
}

impl Document {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::default(),
            path: None,
            modified: false,
            language: Language::PlainText,
            syntax_engine: None,
            version: 1,
            history: History::new(1000),
        }
    }

    pub fn from_file(path: &Path) -> Self {
        let content = std::fs::read_to_string(path).unwrap_or_default();

        let language = Language::from_path(path);
        let syntax_engine = SyntaxEngine::new(&language).ok();

        Self {
            buffer: Buffer::from_string(&content),
            path: Some(path.to_path_buf()),
            modified: false,
            language,
            syntax_engine,
            version: 1,
            history: History::new(1000),
        }
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
        self.path.as_ref().map(|p| current.join(p))
    }

    pub fn uri(&self) -> Option<String> {
        self.full_file_path()
            .and_then(|p| p.to_str().map(|s| s.to_string()))
            .map(|s| format!("file://{}", s))
    }

    pub fn get_undo(&mut self) -> Result<Edit> {
        if let Some(change) = self.history.undo() {
            Ok(change)
        } else {
            Err(anyhow::anyhow!("No changes to undo"))
        }
    }

    pub fn get_redo(&mut self) -> Result<Edit> {
        if let Some(change) = self.history.redo() {
            Ok(change)
        } else {
            Err(anyhow::anyhow!("No changes to redo"))
        }
    }
}

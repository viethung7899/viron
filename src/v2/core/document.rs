use crate::core::buffer::Buffer;
use crate::core::syntax::LanguageType;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Document {
    pub buffer: Buffer,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub language: LanguageType,
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .context(format!("Failed to read file: {}", path.display()))?;

        let language = LanguageType::from_path(path);

        Ok(Self {
            buffer: Buffer::from_string(&content),
            path: Some(path.to_path_buf()),
            modified: false,
            language,
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
}

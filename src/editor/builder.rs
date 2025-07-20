use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::config::Config;
use crate::editor::Editor;

#[derive(Default)]
pub struct EditorBuilder {
    pub(super) config: Option<Config>,
    pub(super) file: Option<PathBuf>,
}

impl EditorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_file(mut self, file: impl AsRef<Path>) -> Self {
        self.file = Some(file.as_ref().to_path_buf());
        self
    }

    pub async fn build(self) -> Result<Editor> {
        Editor::from_builder(self).await
    }
}
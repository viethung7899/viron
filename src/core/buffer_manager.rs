use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::buffer::Buffer;
use crate::core::document::Document;
use crate::core::register::RegisterManager;

pub struct BufferManager {
    documents: Vec<Document>,
    current_index: usize,
    path_to_index: HashMap<PathBuf, usize>,
    pub register_manager: RegisterManager
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            current_index: 0,
            path_to_index: HashMap::new(),
            register_manager: RegisterManager::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    // Get the current active document
    pub fn current(&self) -> &Document {
        &self.documents[self.current_index]
    }

    // Get the mutable current document
    pub fn current_mut(&mut self) -> &mut Document {
        &mut self.documents[self.current_index]
    }

    /// Get the current active buffer
    pub fn current_buffer(&self) -> &Buffer {
        &self.current().buffer
    }

    /// Get mutable reference to the current active buffer
    pub fn current_buffer_mut(&mut self) -> &mut Buffer {
        &mut self.current_mut().buffer
    }

    /// Open a file and add it to the buffer list
    pub fn open_file(&mut self, path: &Path) -> usize {
        let mut absolute_path = std::env::current_dir().unwrap_or_default();
        absolute_path.push(path);

        // Check if file is already open
        if let Some(&index) = self.path_to_index.get(&absolute_path) {
            self.current_index = index;
            return index;
        }

        // Load the document
        let document = Document::from_file(path);

        // Add to documents list
        let index = self.documents.len();
        self.documents.push(document);

        // Update path mapping
        self.path_to_index.insert(absolute_path, index);

        // Set as current
        self.current_index = index;

        index
    }

    /// Save the current buffer to its file
    pub fn save_current(&mut self) -> Result<String> {
        let document = self.current_mut();
        document.save()?;
        document.file_name().context("No file name")
    }

    /// Save the current buffer to a specific path
    pub fn save_current_as(&mut self, path: &Path) -> Result<String> {
        let document = self.current_mut();
        document.save_as(path)?;

        // Update path mapping
        self.path_to_index
            .insert(path.to_path_buf(), self.current_index);

        Ok(format!("Saved as {}", path.display()))
    }

    /// Create a new empty buffer
    pub fn new_buffer(&mut self) -> usize {
        let document = Document::new();
        let index = self.documents.len();
        self.documents.push(document);
        self.current_index = index;
        index
    }

    /// Close the current buffer
    pub fn close_current(&mut self) -> Document {
        // Remove from path mapping if it has a path
        let document = self.documents.remove(self.current_index);

        if let Some(path) = document.full_file_path() {
            self.path_to_index.remove(&path);
        }

        // Update indices in the path_to_index map
        for index in self.path_to_index.values_mut() {
            if *index > self.current_index {
                *index -= 1;
            }
        }

        // Update current index
        if self.current_index >= self.documents.len() {
            self.current_index = self.documents.len().saturating_sub(1);
        }

        document
    }

    /// Switch to the next buffer
    pub fn next_buffer(&mut self) {
        if !self.documents.is_empty() {
            self.current_index = (self.current_index + 1) % self.documents.len();
        }
    }

    /// Switch to the previous buffer
    pub fn previous_buffer(&mut self) {
        if !self.documents.is_empty() {
            self.current_index = if self.current_index == 0 {
                self.documents.len() - 1
            } else {
                self.current_index - 1
            };
        }
    }

    /// Switch to a specific buffer by index
    pub fn switch_to(&mut self, index: usize) -> Result<()> {
        if index >= self.documents.len() {
            return Err(anyhow::anyhow!("Invalid buffer index"));
        }
        self.current_index = index;
        Ok(())
    }

    /// Get list of all open buffers
    pub fn list_buffers(&self) -> Vec<BufferInfo> {
        self.documents
            .iter()
            .enumerate()
            .map(|(i, doc)| BufferInfo {
                index: i,
                name: doc.file_name().unwrap_or_else(|| "[No Name]".to_string()),
                path: doc.path.clone(),
                is_current: i == self.current_index,
                is_modified: doc.modified,
            })
            .collect()
    }
}

/// Information about a buffer for display purposes
pub struct BufferInfo {
    pub index: usize,
    pub name: String,
    pub path: Option<PathBuf>,
    pub is_current: bool,
    pub is_modified: bool,
}

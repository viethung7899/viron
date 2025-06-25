use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::core::buffer::Buffer;
use crate::core::document::Document;

pub struct BufferManager {
    // All open documents
    documents: Vec<Document>,
    // Index of the currently active document
    current_index: usize,
    // Map from file paths to document indices for quick lookup
    path_to_index: HashMap<PathBuf, usize>,
}

impl BufferManager {
    pub fn new() -> Self {
        // Create a default empty document
        let mut documents = Vec::new();
        documents.push(Document::new());

        Self {
            documents,
            current_index: 0,
            path_to_index: HashMap::new(),
        }
    }

    /// Get the current active document
    pub fn current(&self) -> &Document {
        &self.documents[self.current_index]
    }

    /// Get mutable reference to the current active document
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
    pub fn open_file(&mut self, path: &Path) -> Result<usize> {
        // Check if file is already open
        if let Some(&index) = self.path_to_index.get(path) {
            self.current_index = index;
            return Ok(index);
        }

        // Load the document
        let document = Document::from_file(path)
            .context(format!("Failed to open file: {}", path.display()))?;

        // Add to documents list
        let index = self.documents.len();
        self.documents.push(document);

        // Update path mapping
        self.path_to_index.insert(path.to_path_buf(), index);

        // Set as current
        self.current_index = index;

        Ok(index)
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
    pub fn close_current(&mut self) -> Result<()> {
        if self.documents.len() <= 1 {
            return Err(anyhow::anyhow!("Cannot close the last buffer"));
        }

        // Remove from path mapping if it has a path
        if let Some(path) = self.current().path.clone() {
            self.path_to_index.remove(&path);
        }

        // Remove the document
        self.documents.remove(self.current_index);

        // Update indices in the path_to_index map
        self.path_to_index = self
            .path_to_index
            .iter()
            .map(|(path, &index)| {
                let new_index = if index > self.current_index {
                    index - 1
                } else {
                    index
                };
                (path.clone(), new_index)
            })
            .collect();

        // Update current index
        if self.current_index >= self.documents.len() {
            self.current_index = self.documents.len() - 1;
        }

        Ok(())
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

use anyhow::Result;
use std::fmt::Debug;
use std::path::PathBuf;

use crate::input::actions::{Action, ActionContext, ActionDefinition, ActionResult};

#[derive(Debug)]
pub struct NextBuffer;

impl Action for NextBuffer {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.buffer_manager.next_buffer();
        Ok(())
    }

    fn describe(&self) -> &str {
        "Switch to next buffer"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::NextBuffer
    }
}

#[derive(Debug)]
pub struct PreviousBuffer;

impl Action for PreviousBuffer {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.buffer_manager.previous_buffer();
        Ok(())
    }

    fn describe(&self) -> &str {
        "Switch to previous buffer"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::PreviousBuffer
    }
}

#[derive(Debug)]
pub struct OpenBuffer {
    path: PathBuf,
}

impl OpenBuffer {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Action for OpenBuffer {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        ctx.buffer_manager.open_file(&self.path)?;
        Ok(())
    }

    fn describe(&self) -> &str {
        "Open buffer from file"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::OpenBuffer {
            path: self.path.to_string_lossy().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct QuitEditor;

impl Action for QuitEditor {
    fn execute(&self, ctx: &mut ActionContext) -> ActionResult {
        // Access to the editor's running state
        *ctx.running = false;
        Ok(())
    }

    fn describe(&self) -> &str {
        "Quit the editor"
    }

    fn to_serializable(&self) -> ActionDefinition {
        ActionDefinition::Quit
    }
}

pub fn open_buffer(path: PathBuf) -> Box<dyn Action> {
    Box::new(OpenBuffer::new(path))
}

pub fn next_buffer() -> Box<dyn Action> {
    Box::new(NextBuffer)
}

pub fn previous_buffer() -> Box<dyn Action> {
    Box::new(PreviousBuffer)
}

pub fn quit_editor() -> Box<dyn Action> {
    Box::new(QuitEditor)
}

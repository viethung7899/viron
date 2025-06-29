use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::time::{Duration, Instant};

// Handle input events from the terminal
pub struct EventHandler {}

impl EventHandler {
    pub fn new() -> Self {
        Self {}
    }

    /// Poll for events, returning a tick if no events are available
    pub fn next(&mut self) -> Result<InputEvent> {
        match event::read()? {
            Event::Key(key) => Ok(InputEvent::Key(key)),
            Event::Resize(width, height) => Ok(InputEvent::Resize(width, height)),
            _ => Ok(InputEvent::None),
        }
    }
}

// Possible input events
#[derive(Debug)]
pub enum InputEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    None,
}

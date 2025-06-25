use anyhow::Result;
use crossterm::event::{self, Event, KeyEvent};
use std::time::{Duration, Instant};

// Handle input events from the terminal
pub struct EventHandler {
    timeout: Duration,
    last_tick: Instant,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            timeout: tick_rate,
            last_tick: Instant::now(),
        }
    }

    /// Poll for events, returning a tick if no events are available
    pub fn next(&mut self) -> Result<InputEvent> {
        let timeout = self
            .timeout
            .checked_sub(self.last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => Ok(InputEvent::Key(key)),
                Event::Resize(width, height) => Ok(InputEvent::Resize(width, height)),
                _ => Ok(InputEvent::None),
            }
        } else {
            self.last_tick = Instant::now();
            Ok(InputEvent::Tick)
        }
    }
}

// Possible input events
pub enum InputEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
    None,
}

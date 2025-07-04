use std::time::Duration;

use crossterm::event::{Event, EventStream, KeyEvent};
use futures::{FutureExt, StreamExt};
use tokio::time::{interval, Interval};

// Handle input events from the terminal
pub struct EventHandler {
    event_stream: EventStream,
    tick_interval: Interval,
}

impl EventHandler {
    pub fn new() -> Self {
        Self {
            event_stream: EventStream::new(),
            tick_interval: interval(Duration::from_millis(500)),
        }
    }

    /// Poll for events, returning a tick if no events are available
    pub async fn next(&mut self) -> anyhow::Result<InputEvent> {
        tokio::select! {
            event = self.event_stream.next().fuse() => {
                match event {
                    Some(Ok(event)) => match event {
                        Event::Key(key_event) => Ok(InputEvent::Key(key_event)),
                        Event::Resize(width, height) => Ok(InputEvent::Resize(width, height)),
                        _ => Ok(InputEvent::None), // Ignore other events for now
                    }
                    Some(Err(e)) => Err(anyhow::anyhow!("Error reading event: {}", e)),
                    None => Ok(InputEvent::None), // Stream closed
                }
            }
            _ = self.tick_interval.tick().fuse() => {
                Ok(InputEvent::Tick)
            }
        }
    }
}

// Possible input events
#[derive(Debug)]
pub enum InputEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
    None,
}

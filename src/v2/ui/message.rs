use anyhow::Result;
use crossterm::{
    cursor, queue,
    style::{Color, PrintStyledContent, Stylize},
};
use std::io::Write;
use std::time::{Duration, Instant};

pub struct MessageArea {
    message: Option<(String, Instant, Duration, MessageType)>,
    width: usize,
}

pub enum MessageType {
    Info,
    Error,
    Warning,
}

impl MessageArea {
    pub fn new(width: usize) -> Self {
        Self {
            message: None,
            width,
        }
    }

    pub fn set_message(&mut self, message: &str, message_type: MessageType) {
        let duration = match message_type {
            MessageType::Error => Duration::from_secs(5),
            _ => Duration::from_secs(3),
        };

        self.message = Some((message.to_string(), Instant::now(), duration, message_type));
    }

    pub fn render<W: Write>(&mut self, writer: &mut W, screen_height: usize) -> Result<()> {
        if let Some((ref message, created, duration, ref msg_type)) = self.message {
            if created.elapsed() > duration {
                self.message = None;
                return Ok(());
            }

            // Position at the bottom of the screen, above status line
            let row = screen_height - 2;
            queue!(writer, cursor::MoveTo(0, row as u16))?;

            // Truncate message if needed
            let display_msg = if message.len() > self.width {
                &message[..self.width]
            } else {
                message
            };

            // Choose color based on message type
            let styled_msg = match msg_type {
                MessageType::Info => display_msg.stylize().with(Color::White),
                MessageType::Error => display_msg.stylize().with(Color::Red),
                MessageType::Warning => display_msg.stylize().with(Color::Yellow),
            };

            queue!(writer, PrintStyledContent(styled_msg))?;
        }

        Ok(())
    }

    pub fn has_message(&self) -> bool {
        self.message.is_some()
    }

    pub fn clear(&mut self) {
        self.message = None;
    }
}

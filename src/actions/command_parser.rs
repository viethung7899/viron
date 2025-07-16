use crate::actions::core::{CompositeExecutable, Executable};
use crate::actions::types::{buffer, movement};
use anyhow::{Context, Result, anyhow};
use std::path::PathBuf;

pub fn parse_command(input: &str) -> Result<Box<dyn Executable>> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    let command = parts[0];

    match command.to_lowercase().as_str() {
        "q" | "quit" => {
            let force = parts.get(1).map_or(false, |&arg| arg == "!");
            Ok(Box::new(buffer::CloseBuffer::force(force)))
        }
        "q!" | "quit!" => Ok(Box::new(buffer::CloseBuffer::force(true))),
        "w" | "write" => {
            let path = parts.get(1).map(|&s| PathBuf::from(s));
            Ok(Box::new(buffer::WriteBuffer::new(path)))
        }
        "wq" | "writequit" => {
            let path = parts.get(1).map(|&s| PathBuf::from(s));
            let mut executable = CompositeExecutable::new();
            executable
                .add(buffer::WriteBuffer::new(path))
                .add(buffer::CloseBuffer::force(false));
            Ok(Box::new(executable))
        }
        "e" | "edit" => {
            let path = parts
                .get(1)
                .map(|&s| PathBuf::from(s))
                .context("No such file or directory")?;
            Ok(Box::new(buffer::OpenBuffer::new(path)))
        }
        "bn" | "bnext" => Ok(Box::new(buffer::NextBuffer)),
        "bp" | "bprevious" => Ok(Box::new(buffer::PreviousBuffer)),
        cmd => {
            if let Ok(line_number) = cmd.parse::<usize>() {
                Ok(Box::new(movement::GoToLine::new(
                    line_number.saturating_sub(1),
                )))
            } else {
                Err(anyhow!("Command not found {}", input))
            }
        }
    }
}

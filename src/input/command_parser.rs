use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

use crate::input::actions::{
    CloseBuffer, CompositeExecutable, Executable, GoToLine, NextBuffer, OpenBuffer, PreviousBuffer,
    WriteBuffer,
};

pub fn parse_command(input: &str) -> Result<Box<dyn Executable>> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    let command = parts[0];

    match command.to_lowercase().as_str() {
        "q" | "quit" => {
            let force = parts.get(1).map_or(false, |&arg| arg == "!");
            Ok(Box::new(CloseBuffer::force(force)))
        }
        "q!" | "quit!" => Ok(Box::new(CloseBuffer::force(true))),
        "w" | "write" => {
            let path = parts.get(1).map(|&s| PathBuf::from(s));
            Ok(Box::new(WriteBuffer::new(path)))
        }
        "wq" | "writequit" => {
            let path = parts.get(1).map(|&s| PathBuf::from(s));
            let mut executable = CompositeExecutable::new();
            executable
                .add(WriteBuffer::new(path))
                .add(CloseBuffer::force(false));
            Ok(Box::new(executable))
        }
        "e" | "edit" => {
            let path = parts
                .get(1)
                .map(|&s| PathBuf::from(s))
                .context("No such file or directory")?;
            Ok(Box::new(OpenBuffer::new(path)))
        }
        "bn" | "bnext" => Ok(Box::new(NextBuffer)),
        "bp" | "bprevious" => Ok(Box::new(PreviousBuffer)),
        cmd => {
            if let Ok(line_number) = cmd.parse::<usize>() {
                Ok(Box::new(GoToLine::new(line_number.saturating_sub(1))))
            } else {
                Err(anyhow!("Command not found {}", input))
            }
        }
    }
}

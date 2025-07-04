use anyhow::{anyhow, Result};

use crate::input::actions::{create_action_from_definition, ActionDefinition, Executable};

pub fn parse_command(input: &str) -> Result<Box<dyn Executable>> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    let command = parts[0];

    let definition = match command.to_lowercase().as_str() {
        "q" | "quit" => Ok(ActionDefinition::CloseBuffer {
            force: parts.get(1).map_or(false, |&arg| arg == "!"),
        }),
        "q!" | "quit!" => Ok(ActionDefinition::CloseBuffer { force: true }),
        "w" | "write" => {
            let path = parts.get(1).map(|&s| s.to_string());
            Ok(ActionDefinition::WriteBuffer { path })
        }
        "wq" | "writequit" => {
            let path = parts.get(1).map(|&s| s.to_string());
            Ok(ActionDefinition::Composite {
                description: "Write and quit".to_string(),
                actions: vec![
                    ActionDefinition::WriteBuffer { path },
                    ActionDefinition::CloseBuffer { force: false },
                ],
            })
        }
        cmd => {
            if let Ok(line_number) = cmd.parse::<usize>() {
                Ok(ActionDefinition::GoToLine { line_number: line_number.saturating_sub(1) })
            } else {
                Err(anyhow!("Command not found {}", input))
            }
        }
    }?;

    Ok(create_action_from_definition(&definition))
}

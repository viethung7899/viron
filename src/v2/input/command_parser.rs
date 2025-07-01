use anyhow::{anyhow, Result};

use crate::input::actions::{create_action_from_definition, Action, ActionDefinition};

pub fn parse_command(input: &str) -> Result<Box<dyn Action>> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    if parts.is_empty() {
        return Err(anyhow!("Empty command"));
    }

    let command = parts[0];

    let definition = match command.to_lowercase().as_str() {
        "q" | "quit" => Ok(ActionDefinition::Quit {
            force: parts.get(1).map_or(false, |&arg| arg == "!"),
        }),
        "q!" | "quit!" => Ok(ActionDefinition::Quit { force: true }),
        "w" | "write" => {
            let path = parts.get(1).map(|&s| s.to_string());
            Ok(ActionDefinition::WriteBuffer { path })
        },
        "wq" | "writequit" => {
            let path = parts.get(1).map(|&s| s.to_string());
            Ok(ActionDefinition::Composite {
                description: "Write and quit".to_string(),
                actions: vec![
                    ActionDefinition::WriteBuffer { path },
                    ActionDefinition::Quit { force: false },
                ],
            })
        },
        _ => Err(anyhow!("Command not found {}", input)),
    }?;

    Ok(create_action_from_definition(&definition))
}

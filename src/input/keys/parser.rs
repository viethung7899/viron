use crossterm::event::{KeyCode, KeyModifiers};
use nom::{multi, branch, IResult, Parser};
use nom::bytes::tag;
use nom::character::satisfy;
use anyhow::{anyhow, Result};
use nom::combinator::complete;
use crate::input::keys::KeyEvent;
use crate::input::keys::sequence::KeySequence;

fn keycode(input: &str) -> IResult<&str, KeyCode> {
    branch::alt((
        tag("<Backspace>").map(|_| KeyCode::Backspace),
        tag("<Enter>").map(|_| KeyCode::Enter),
        tag("<Left>").map(|_| KeyCode::Left),
        tag("<Right>").map(|_| KeyCode::Right),
        tag("<Up>").map(|_| KeyCode::Up),
        tag("<Down>").map(|_| KeyCode::Down),
        tag("<Home>").map(|_| KeyCode::Home),
        tag("<End>").map(|_| KeyCode::End),
        tag("<PageUp>").map(|_| KeyCode::PageUp),
        tag("<PageDown>").map(|_| KeyCode::PageDown),
        tag("<Tab>").map(|_| KeyCode::Tab),
        tag("<Delete>").map(|_| KeyCode::Delete),
        tag("<Esc>").map(|_| KeyCode::Esc),
        tag("<lt>").map(|_| KeyCode::Char('<')),
        tag("<gt>").map(|_| KeyCode::Char('>')),
        satisfy(|c| c != '<' && c != '>').map(|c| KeyCode::Char(c))
    )).parse(input)
}

fn key_event_with_modifiers(input: &str) -> IResult<&str, KeyEvent> {
    let (input, modifiers) = branch::alt((
        tag("<C-").map(|_| KeyModifiers::CONTROL),
        tag("<A-").map(|_| KeyModifiers::ALT),
    )).parse(input)?;

    let (input, code) = keycode(input)?;
    let (input, _) = tag(">").parse(input)?;
    Ok((input, KeyEvent { code, modifiers }))
}

fn key_event(input: &str) -> IResult<&str, KeyEvent> {
    branch::alt((
        key_event_with_modifiers,
        keycode.map(|code| {
            let modifiers = match code {
                KeyCode::Char(c) if c.is_uppercase() => KeyModifiers::SHIFT,
                _ => KeyModifiers::NONE
            };
            KeyEvent { code, modifiers }
        }),
    )).parse(input)
}

fn key_sequence(input: &str) -> IResult<&str, KeySequence> {
    let (input, keys) = multi::many0(complete(key_event)).parse(input)?;
    Ok((input, KeySequence::from_keys(keys)))
}

pub fn parse_key_sequence(input: &str) -> Result<KeySequence> {
    let result = key_sequence(input);
    match result {
        Ok((remain, sequence)) => if remain.is_empty() {
            Ok(sequence)
        } else {
            Err(anyhow!("Unexpected input after key sequence: '{}'", remain))
        },
        _ => Err(anyhow!("Failed to parse key sequence")),
    }
}
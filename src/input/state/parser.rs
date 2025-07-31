use crate::actions::core::ActionDefinition;
use crate::core::mode::Mode;
use crate::core::register::RegisterName;
use crate::input::keymaps::KeyMap;
use nom::IResult;
use nom::Parser;
use nom::bytes::complete::tag;
use nom::character::anychar;
use nom::combinator::opt;

pub struct ParserResult<T> {
    pub result: T,
    pub length: usize,
}

pub fn register(input: &str) -> IResult<&str, ParserResult<RegisterName>> {
    let (input, _) = tag("\"")(input)?;
    let (input, char) = anychar(input)?;
    let register = RegisterName::from_char(char).map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
    })?;

    Ok((
        input,
        ParserResult {
            result: register,
            length: 2,
        },
    ))
}

fn positive_count(input: &str) -> IResult<&str, ParserResult<usize>> {
    let (input, num_str) = nom::character::complete::digit1(input)?;
    let num = num_str.parse::<usize>().map_err(|_| {
        nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
    })?;
    Ok((
        input,
        ParserResult {
            result: num,
            length: num_str.len(),
        },
    ))
}

fn from_keymap(
    mode: &Mode,
    keymap: &KeyMap,
) -> impl Fn(&str) -> IResult<&str, ParserResult<ActionDefinition>> {
    move |input: &str| {
        if let Some(action) = keymap.get_action(mode, input) {
            Ok((
                "",
                ParserResult {
                    result: action.clone(),
                    length: input.len(),
                },
            ))
        } else if keymap.is_partial_match(mode, input) {
            Err(nom::Err::Incomplete(nom::Needed::Unknown))
        } else {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        }
    }
}

pub fn from_keymap_with_repeat(
    mode: &Mode,
    keymap: &KeyMap,
) -> impl Fn(&str) -> IResult<&str, ParserResult<(Option<usize>, ActionDefinition)>> {
    move |input: &str| {
        let (input, repeat) = opt(positive_count).parse(input)?;
        let (input, action) = from_keymap(mode, keymap)(input)?;
        let length = action.length + repeat.as_ref().map_or(0, |r| r.length);
        let result = ParserResult {
            result: (repeat.map(|r| r.result), action.result),
            length,
        };
        Ok((input, result))
    }
}

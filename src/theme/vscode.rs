use std::collections::HashMap;

use anyhow::{Error, Result, bail};
use crossterm::style::Color;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::theme::{Style, Theme, TokenStyle};

static TRANSLATION_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("constant", "constant"),
        ("entity.name.type", "type"),
        ("support.type", "type"),
        ("entity.name.function.constructor", "constructor"),
        ("variable.other.enummember", "constructor"),
        ("entity.name.function", "function"),
        ("meta.function-call", "function"),
        ("entity.name.function.member", "function.method"),
        ("variable.function", "function.method"),
        ("entity.name.function.macro", "function.macro"),
        ("support.function.macro", "function.macro"),
        ("variable.other.member", "property"),
        ("variable.other.property", "property"),
        ("variable.parameter", "variable.parameter"),
        ("entity.name.label", "label"),
        ("comment", "comment"),
        ("punctuation.definition.comment", "comment"),
        ("punctuation.section.block", "punctuation.bracket"),
        ("punctuation.definition.brackets", "punctuation.bracket"),
        ("punctuation.separator", "punctuation.delimiter"),
        ("punctuation.accessor", "punctuation.delimiter"),
        ("keyword", "keyword"),
        ("keyword.control", "keyword"),
        ("support.type.primitive", "type.builtin"),
        ("keyword.type", "type.builtin"),
        ("variable.language", "variable.builtin"),
        ("support.variable", "variable.builtin"),
        ("string.quoted.double", "string"),
        ("string.quoted.single", "string"),
        ("constant.language", "constant.builtin"),
        ("constant.numeric", "constant.builtin"),
        ("constant.character", "constant.builtin"),
        ("constant.character.escape", "escape"),
        ("keyword.operator", "operator"),
        ("storage.modifier.attribute", "attribute"),
        ("meta.attribute", "attribute"),
    ])
});

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VsCodeTokenColor {
    name: Option<String>,
    scope: VsCodeScope,
    settings: VsCodeTokenColorSettings,
}

impl TryFrom<VsCodeTokenColor> for TokenStyle {
    type Error = Error;

    fn try_from(value: VsCodeTokenColor) -> std::result::Result<Self, Self::Error> {
        Ok(TokenStyle {
            name: value.name,
            scope: value.scope.into(),
            style: Style::try_from(value.settings)?,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VsCodeTokenColorSettings {
    foreground: Option<String>,
    background: Option<String>,
    font_style: Option<String>,
}

impl TryFrom<VsCodeTokenColorSettings> for Style {
    type Error = Error;

    fn try_from(value: VsCodeTokenColorSettings) -> std::result::Result<Self, Self::Error> {
        let foreground = if let Some(fg) = value.foreground {
            Some(parse_rgb(&fg)?)
        } else {
            None
        };
        let background = if let Some(bg) = value.background {
            Some(parse_rgb(&bg)?)
        } else {
            None
        };
        let bold = value
            .font_style
            .as_ref()
            .map_or(false, |v| v.contains("bold"));
        let italic = value
            .font_style
            .as_ref()
            .map_or(false, |v| v.contains("italic"));
        Ok(Style {
            foreground,
            background,
            bold,
            italic,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VsCodeTheme {
    name: Option<String>,
    #[serde(rename = "type")]
    type_: Option<String>,
    colors: HashMap<String, String>,
    token_colors: Vec<VsCodeTokenColor>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(super) enum VsCodeScope {
    Single(String),
    Multiple(Vec<String>),
}

impl From<VsCodeScope> for Vec<String> {
    fn from(value: VsCodeScope) -> Self {
        match value {
            VsCodeScope::Single(s) => vec![translate_scope(&s)],
            VsCodeScope::Multiple(v) => v.into_iter().map(|s| translate_scope(&s)).collect(),
        }
    }
}

fn parse_rgb(s: &str) -> Result<Color> {
    if !s.starts_with('#') {
        bail!("Invalid hex string");
    }

    let s = &s[1..];
    if s.len() != 6 {
        bail!("Invalid hex string, got #{s}");
    }

    let r = u8::from_str_radix(&s[0..2], 16)?;
    let g = u8::from_str_radix(&s[2..4], 16)?;
    let b = u8::from_str_radix(&s[4..6], 16)?;

    Ok(Color::Rgb { r, g, b })
}

fn translate_scope(key: &str) -> String {
    TRANSLATION_MAP
        .get(key)
        .map_or(key.to_string(), |s| s.to_string())
}

pub fn parse_vscode_theme(file: &str) -> Result<Theme> {
    let content = std::fs::read_to_string(file)?;
    let vscode: VsCodeTheme = serde_json::from_str(&content)?;
    println!("{vscode:#?}");

    let name = vscode.name.unwrap_or_default();
    let editor_foreground = vscode
        .colors
        .get("editor.foreground")
        .and_then(|s| parse_rgb(s.as_str()).ok());
    let editor_background = vscode
        .colors
        .get("editor.background")
        .and_then(|s| parse_rgb(s.as_str()).ok());

    let token_styles = vscode
        .token_colors
        .into_iter()
        .map(|tc| tc.try_into())
        .collect::<Result<Vec<TokenStyle>, _>>()?;

    Ok(Theme {
        name,
        editor_style: Style {
            foreground: editor_foreground,
            background: editor_background,
            ..Style::default()
        },
        token_styles,
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_vscode() {
        let theme = parse_vscode_theme("./src/themes/catppuchin/mocha.json").unwrap();
        println!("Theme: {theme:?}");
    }
}

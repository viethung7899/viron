use std::{collections::HashMap, u8};

use anyhow::{Error, Result, bail};
use crossterm::style::Color;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::v1::theme::{DiagnosticStyles, StatusLineStyle, Style, Theme, TokenStyle};

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
    scope: VsCodeScope,
    settings: VsCodeTokenColorSettings,
}

impl TryFrom<VsCodeTokenColor> for TokenStyle {
    type Error = Error;

    fn try_from(value: VsCodeTokenColor) -> std::result::Result<Self, Self::Error> {
        Ok(TokenStyle {
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
    colors: HashMap<String, String>,
    token_colors: Vec<VsCodeTokenColor>,
}

impl VsCodeTheme {
    pub fn get_color(&self, key: &str) -> Option<Color> {
        self.colors
            .get(key)
            .and_then(|s| parse_rgb(s.as_str()).ok())
    }

    pub fn get_color_with_alpha(&self, key: &str, background: Option<&Color>) -> Option<Color> {
        self.colors
            .get(key)
            .and_then(|s| parse_rgba(s.as_str(), background).ok())
    }
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

impl From<&VsCodeTheme> for StatusLineStyle {
    fn from(theme: &VsCodeTheme) -> Self {
        let inner_background = theme.get_color("statusBar.background");
        let inner_foreground = theme.get_color("statusBar.foreground");

        let outer_foreground = theme.get_color("statusBar.background");
        let normal_background = theme.get_color("terminal.ansiBlue");
        let insert_background = theme.get_color("terminal.ansiGreen");
        let command_background = theme.get_color("terminal.ansiYellow");

        let inner = Style {
            background: inner_background,
            foreground: inner_foreground,
            bold: true,
            ..Style::default()
        };

        let normal = Style {
            background: normal_background,
            foreground: outer_foreground,
            bold: true,
            ..Style::default()
        };

        let insert = Style {
            background: insert_background,
            foreground: outer_foreground,
            bold: true,
            ..Style::default()
        };

        let command = Style {
            background: command_background,
            foreground: outer_foreground,
            bold: true,
            ..Style::default()
        };

        StatusLineStyle {
            normal,
            insert,
            command,
            inner,
        }
    }
}

impl From<&VsCodeTheme> for DiagnosticStyles {
    fn from(theme: &VsCodeTheme) -> Self {
        let background = theme.get_color("editor.background");

        let error = Style {
            foreground: theme.get_color("errorLens.errorForeground"),
            background: theme
                .get_color_with_alpha("errorLens.errorBackground", background.as_ref()),
            italic: true,
            ..Style::default()
        };

        let hint = Style {
            foreground: theme.get_color("errorLens.hintForeground"),
            background: theme.get_color_with_alpha("errorLens.hintBackground", background.as_ref()),
            italic: true,
            ..Style::default()
        };

        let warning = Style {
            foreground: theme.get_color("errorLens.warningForeground"),
            background: theme
                .get_color_with_alpha("errorLens.warningBackground", background.as_ref()),
            italic: true,
            ..Style::default()
        };

        let info = Style {
            foreground: theme.get_color("errorLens.infoForeground"),
            background: theme.get_color_with_alpha("errorLens.infoBackground", background.as_ref()),
            italic: true,
            ..Style::default()
        };

        DiagnosticStyles {
            error,
            hint,
            warning,
            info,
        }
    }
}

fn parse_rgb(s: &str) -> Result<Color> {
    parse_rgba(s, None)
}

fn parse_rgba(s: &str, background: Option<&Color>) -> Result<Color> {
    let Some(color) = s.strip_prefix("#") else {
        bail!("Invalid hex string");
    };

    if color.len() != 6 && color.len() != 8 {
        bail!("Invalid hex string, got #{color}");
    }

    let r = u8::from_str_radix(&color[0..2], 16)?;
    let g = u8::from_str_radix(&color[2..4], 16)?;
    let b = u8::from_str_radix(&color[4..6], 16)?;

    let Some(Color::Rgb {
        r: bg_r,
        g: bg_g,
        b: bg_b,
    }) = background
    else {
        return Ok(Color::Rgb { r, g, b });
    };

    let a = if color.len() == 8 {
        u8::from_str_radix(&s[6..8], 16)?
    } else {
        u8::MAX
    };

    if a == u8::MAX {
        return Ok(Color::Rgb { r, g, b });
    };

    let alpha = a as f32 / 255.0;
    let [r, g, b] = [(r, bg_r), (g, bg_g), (b, bg_b)].map(|(fg, bg)| {
        let fg = fg as f32 * alpha;
        let bg = *bg as f32 * (1.0 - alpha);
        (fg + bg).floor() as u8
    });

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

    let editor_foreground = vscode.get_color("editor.foreground");
    let editor_background = vscode.get_color("editor.background");
    let gutter_foreground = vscode.get_color("editorLineNumber.foreground");
    let gutter_background = vscode.get_color("editorLineNumber.background");

    let status_line_style = StatusLineStyle::from(&vscode);
    let diagnostic_styles = DiagnosticStyles::from(&vscode);

    let token_styles = vscode
        .token_colors
        .into_iter()
        .map(|tc| tc.try_into())
        .collect::<Result<Vec<TokenStyle>, _>>()?;

    Ok(Theme {
        editor_style: Style {
            foreground: editor_foreground,
            background: editor_background,
            ..Style::default()
        },
        gutter_style: Style {
            foreground: gutter_foreground,
            background: gutter_background,
            ..Style::default()
        },
        status_line_style,
        token_styles,
        diagnostic_styles,
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

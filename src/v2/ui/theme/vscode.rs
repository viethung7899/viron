use std::{collections::HashMap, u8};

use anyhow::{Result, bail};
use crossterm::style::Color;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::ui::theme::Style;

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
    pub(super) scope: VsCodeScope,
    pub(super) settings: VsCodeTokenColorSettings,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VsCodeTokenColorSettings {
    pub(super) foreground: Option<String>,
    pub(super) background: Option<String>,
    pub(super) font_style: Option<String>,
}

impl VsCodeTokenColorSettings {
    pub fn to_style(&self) -> Result<Style> {
        let foreground = if let Some(fg) = &self.foreground {
            Some(parse_rgb(&fg)?)
        } else {
            None
        };
        let background = if let Some(bg) = &self.background {
            Some(parse_rgb(&bg)?)
        } else {
            None
        };
        let (bold, italic) = self
            .font_style
            .as_ref()
            .map(|style| (style.contains("bold"), style.contains("italic")))
            .unwrap_or_default();
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
    pub(super) colors: HashMap<String, String>,
    pub(super) token_colors: Vec<VsCodeTokenColor>,
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

impl VsCodeScope {
    pub fn translate(&self) -> Vec<String> {
        match self {
            VsCodeScope::Single(s) => translate_scope(s).map(|s| vec![s]).unwrap_or_default(),
            VsCodeScope::Multiple(items) => {
                items.iter().filter_map(|s| translate_scope(s)).collect()
            }
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

pub fn translate_scope(key: &str) -> Option<String> {
    TRANSLATION_MAP.get(key).map(|s| s.to_string())
}

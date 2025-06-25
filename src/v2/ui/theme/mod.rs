use crossterm::style::{Color, ContentStyle};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub author: Option<String>,
    pub colors: ThemeColors,
    pub syntax: HashMap<String, SyntaxStyle>,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub editor_background: Color,
    pub editor_foreground: Color,
    pub status_background: Color,
    pub status_foreground: Color,
    pub selection_background: Color,
    pub selection_foreground: Color,
    pub gutter_background: Color,
    pub gutter_foreground: Color,
    pub line_highlight: Color,
    pub cursor: Color,
    pub command_foreground: Color,
    pub command_background: Color,
}

#[derive(Debug, Clone)]
pub struct SyntaxStyle {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
}

impl Theme {
    pub fn default_dark() -> Self {
        Self {
            name: "Default Dark".to_string(),
            author: Some("Viron".to_string()),
            colors: ThemeColors {
                editor_background: Color::Black,
                editor_foreground: Color::White,
                status_background: Color::Blue,
                status_foreground: Color::White,
                selection_background: Color::DarkBlue,
                selection_foreground: Color::White,
                gutter_background: Color::Black,
                gutter_foreground: Color::DarkGrey,
                line_highlight: Color::Grey,
                cursor: Color::White,
                command_foreground: Color::White,
                command_background: Color::Black,
            },
            syntax: HashMap::new(),
        }
    }

    pub fn style_for_token(&self, token_type: &str) -> ContentStyle {
        let mut style = ContentStyle::default();

        if let Some(syntax_style) = self.syntax.get(token_type) {
            if let Some(fg) = syntax_style.foreground {
                style.background_color = fg.into();
            }
            if let Some(bg) = syntax_style.background {
                style.background_color = bg.into();
            }
        }

        style
    }
}

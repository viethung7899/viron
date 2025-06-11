use crossterm::style::{Attribute, Attributes, Color, ContentStyle};

mod vscode;

pub use vscode::parse_vscode_theme;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub bold: bool,
    pub italic: bool,
}

impl Style {
    pub fn to_content_style(&self, fallback: Option<&Style>) -> ContentStyle {
        let foreground_color = self
            .foreground
            .or(fallback.and_then(|style| style.foreground));
        let background_color = self
            .background
            .or(fallback.and_then(|style| style.background));
        let mut attributes = Attributes::default();

        if self.italic {
            attributes.set(Attribute::Italic);
        }

        if self.bold {
            attributes.set(Attribute::Bold);
        }

        ContentStyle {
            foreground_color,
            background_color,
            attributes,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TokenStyle {
    pub scope: Vec<String>,
    pub style: Style,
}

#[derive(Debug, Clone, Default)]
pub struct StatusLineStyle {
    pub normal: Style,
    pub insert: Style,
    pub inner: Style,
}

#[derive(Debug, Clone, Default)]
pub struct Theme {
    pub gutter_style: Style,
    pub editor_style: Style,
    pub status_line_style: StatusLineStyle,
    pub token_styles: Vec<TokenStyle>,
}

impl Theme {
    pub fn get_style(&self, scope: &str) -> Style {
        self.token_styles
            .iter()
            .find(|style| style.scope.contains(&scope.to_string()))
            .map(|style| style.style.clone())
            .unwrap_or(self.editor_style.clone())
    }
}

use crossterm::style::Color;

mod vscode;

pub use vscode::parse_vscode_theme;

#[derive(Debug, Clone)]
pub struct Style {
    pub foreground: Color,
    pub background: Color,
    pub bold: bool,
    pub italic: bool,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            foreground: Color::Reset,
            background: Color::Reset,
            bold: false,
            italic: false,
        }
    }
}

#[derive(Debug)]
pub struct TokenStyle {
    pub name: Option<String>,
    pub scope: Vec<String>,
    pub style: Style,
}

#[derive(Debug)]
pub struct Theme {
    pub name: String,
    pub editor_style: Style,
    pub token_styles: Vec<TokenStyle>,
}

impl Theme {
    pub fn get_style(&self, scope: &str) -> Option<Style> {
        self.token_styles.iter().find_map(|style| {
            if style.scope.contains(&scope.to_string()) {
                Some(style.style.clone())
            } else {
                None
            }
        })
    }
}

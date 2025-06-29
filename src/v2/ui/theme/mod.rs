use anyhow::Result;
use crossterm::style::{Attribute, Attributes, Color, Colors, ContentStyle};
use std::collections::HashMap;
use std::fs;
use std::io::BufReader;
use crate::ui::theme::vscode::{VsCodeTheme};

pub mod vscode;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Style {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub bold: bool,
    pub italic: bool,
}

impl From<Colors> for Style {
    fn from(colors: Colors) -> Self {
        Self {
            foreground: colors.foreground,
            background: colors.background,
            ..Default::default()
        }
    }
}

impl Style {
    pub fn to_content_style(&self, fallback: &Style) -> ContentStyle {
        let foreground_color = self
            .foreground
            .or(fallback.foreground);
        let background_color = self
            .background
            .or(fallback.background);
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
pub struct Theme {
    pub colors: ThemeColors,
    pub token_styles: HashMap<String, Style>,
}

#[derive(Debug, Clone)]
pub struct ThemeColors {
    pub editor: Colors,
    pub gutter: Colors,
    pub status: StatusColors,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            editor: default_colors(),
            gutter: default_colors(),
            status: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StatusColors {
    pub normal: Colors,
    pub insert: Colors,
    pub command: Colors,
    pub inner: Colors,
}

impl Default for StatusColors {
    fn default() -> Self {
        Self {
            normal: default_colors(),
            insert: default_colors(),
            command: default_colors(),
            inner: default_colors(),
        }
    }
}

fn default_colors() -> Colors {
    Colors {
        foreground: None,
        background: None,
    }
}

impl Theme {
    pub fn style_for_token(&self, token_type: &str) -> ContentStyle {
        let mut style = ContentStyle::default();

        if let Some(token_style) = self.token_styles.get(token_type) {
            if let Some(fg) = token_style.foreground {
                style.background_color = fg.into();
            }
            if let Some(bg) = token_style.background {
                style.background_color = bg.into();
            }
        }

        style
    }

    pub fn load_from_file(path: impl AsRef<std::path::Path>) -> Result<Theme> {
        let reader = BufReader::new(fs::File::open(path)?);
        let vscode: VsCodeTheme = serde_json::from_reader(reader)?;
        Ok(Theme::from(&vscode))
    }
    
    pub fn editor_style(&self) -> Style {
        Style {
            foreground: self.colors.editor.foreground,
            background: self.colors.editor.background,
            ..Default::default()
        }
    }
}

impl From<&VsCodeTheme> for StatusColors {
    fn from(vscode: &VsCodeTheme) -> Self {
        let inner = Colors {
            background: vscode.get_color("statusBar.background"),
            foreground: vscode.get_color("statusBar.foreground"),
        };

        let outer_foreground = vscode.get_color("statusBar.background");

        let normal = Colors {
            foreground: outer_foreground,
            background: vscode.get_color("terminal.ansiBlue"),
        };

        let edit = Colors {
            foreground: outer_foreground,
            background: vscode.get_color("terminal.ansiGreen"),
        };

        let command = Colors {
            foreground: outer_foreground,
            background: vscode.get_color("terminal.ansiYellow"),
        };

        StatusColors {
            normal,
            insert: edit,
            command,
            inner,
        }
    }
}

impl From<&VsCodeTheme> for ThemeColors {
    fn from(vscode: &VsCodeTheme) -> Self {
        ThemeColors {
            editor: Colors {
                foreground: vscode.get_color("editor.foreground"),
                background: vscode.get_color("editor.background"),
            },
            gutter: Colors {
                foreground: vscode.get_color("editorLineNumber.foreground"),
                background: vscode.get_color("editorLineNumber.background"),
            },
            status: StatusColors::from(vscode),
        }
    }
}

impl From<&VsCodeTheme> for Theme {
    fn from(vscode: &VsCodeTheme) -> Self {
        let colors = ThemeColors::from(vscode);
        let token_colors = &vscode.token_colors;
        let token_styles = token_colors
            .into_iter()
            .filter_map(|token_color| {
                let style: Style = token_color.settings.to_style().ok()?;
                Some(
                    token_color
                        .scope
                        .translate()
                        .iter()
                        .map(|key| (key.clone(), style.clone()))
                        .collect::<Vec<_>>(),
                )
            })
            .flatten()
            .collect();
        Theme {
            colors,
            token_styles,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_file() {
        let theme = Theme::load_from_file("themes/catppuchin/mocha.json").unwrap();
        println!("{:#?}", theme);
    }
}

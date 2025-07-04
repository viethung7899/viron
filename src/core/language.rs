use std::{env, fs, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Language {
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    C,
    Cpp,
    Toml,
    Json,
    Markdown,
    Html,
    Css,
    Bash,
    #[default]
    PlainText, // Fallback for unsupported languages
}

impl Language {
    pub fn is_plain_text(&self) -> bool {
        return self == &Language::PlainText;
    }

    pub fn from_extension(extension: &str) -> Self {
        match extension.to_lowercase().as_str() {
            "rs" => Self::Rust,
            "js" => Self::JavaScript,
            "jsx" => Self::JavaScript,
            "ts" => Self::TypeScript,
            "tsx" => Self::TypeScript,
            "py" => Self::Python,
            "go" => Self::Go,
            "c" => Self::C,
            "cpp" | "cc" | "cxx" | "h" | "hpp" => Self::Cpp,
            "toml" => Self::Toml,
            "json" => Self::Json,
            "md" => Self::Markdown,
            "html" | "htm" => Self::Html,
            "css" => Self::Css,
            "sh" | "bash" => Self::Bash,
            _ => Self::PlainText,
        }
    }

    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(Self::from_extension)
            .unwrap_or(Self::PlainText)
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Python => "python",
            Self::Go => "go",
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Toml => "toml",
            Self::Json => "json",
            Self::Markdown => "markdown",
            Self::Html => "html",
            Self::Css => "css",
            Self::Bash => "bash",
            Self::PlainText => "text",
        }
    }

    pub fn get_tree_sitter_language(&self) -> Option<tree_sitter::Language> {
        match self {
            Self::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            _ => None,
        }
    }

    pub fn get_highlight_query(&self) -> Option<&str> {
        match self {
            Self::Rust => Some(tree_sitter_rust::HIGHLIGHTS_QUERY),
            _ => None,
        }
    }

    pub fn get_language_server(&self) -> Option<&str> {
        let executable = match self {
            Self::Rust => Some("rust-analyzer"),
            _ => None,
        };
        executable.filter(|&executable| is_program_in_path(executable))
    }
}

fn is_program_in_path(program: &str) -> bool {
    if let Ok(path) = env::var("PATH") {
        for p in path.split(":") {
            let p_str = format!("{}/{}", p, program);
            if fs::metadata(p_str).is_ok() {
                return true;
            }
        }
    }
    false
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum RegisterKind {
    #[default]
    Character,
    Line,
}

#[derive(Debug, Default, Clone)]
pub struct Register {
    pub kind: RegisterKind,
    pub content: String,
}

impl Register {
    pub fn new(content: String, kind: RegisterKind) -> Register {
        match kind {
            RegisterKind::Character => Self::character(content),
            RegisterKind::Line => Self::line(content),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    fn character(content: String) -> Register {
        Self {
            content,
            kind: RegisterKind::Character,
        }
    }

    fn line(content: String) -> Register {
        let content = if content.ends_with('\n') {
            content
        } else {
            format!("{}\n", content)
        };
        Self {
            content,
            kind: RegisterKind::Line,
        }
    }
}

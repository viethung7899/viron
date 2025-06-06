#[derive(Debug, Default)]
pub struct Buffer {
    pub file: Option<String>,
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn from_file(file_path: &str) -> Self {
        let lines = std::fs::read_to_string(file_path).unwrap_or_default();
        Self {
            file: Some(file_path.to_string()),
            lines: lines.lines().map(|line| line.to_string()).collect(),
        }
    }

    pub fn get_line(&self, index: usize) -> Option<String> {
        self.lines.get(index).map(|line| line.clone())
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }
}

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

    pub fn insert(&mut self, line: usize, offset: usize, char: char) {
        if let Some(line) = self.lines.get_mut(line) {
            line.insert(offset, char);
        }
    }

    pub fn remove(&mut self, line: usize, offset: usize) {
        if let Some(line) = self.lines.get_mut(line) {
            line.remove(offset);
        }
    }

    pub fn remove_line(&mut self, line: usize) {
        if line < self.len() {
            self.lines.remove(line);
        }
    }
}

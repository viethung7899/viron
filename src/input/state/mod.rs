pub mod internal;
pub mod parser;

#[derive(Debug)]
pub struct InputState {
    sequence: String,
    index: usize,
}

impl InputState {
    pub fn new() -> Self {
        InputState {
            sequence: String::new(),
            index: 0,
        }
    }

    pub fn add_string(&mut self, input: &str) {
        self.sequence.push_str(input);
    }

    pub fn clear(&mut self) {
        self.sequence.clear();
        self.index = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.sequence.is_empty()
    }

    pub fn get_input(&self) -> &str {
        if self.index < self.sequence.len() {
            &self.sequence[self.index..]
        } else {
            ""
        }
    }

    pub fn display(&self) -> &str {
        &self.sequence
    }

    pub fn advance(&mut self, length: usize) {
        self.index += length;
    }
}

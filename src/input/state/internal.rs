#[derive(Debug)]
pub struct RepeatState {
    pub repeat: Option<usize>,
    pub pending_repeat: Option<usize>
}

impl RepeatState {
    pub fn new() -> Self {
        RepeatState {
            repeat: None,
            pending_repeat: None,
        }
    }

    pub fn clear(&mut self) {
        self.repeat = None;
        self.pending_repeat = None;
    }
    
    pub fn push_repeat(&mut self) {
        self.pending_repeat = self.repeat.take();
    }

    pub fn get_total_repeat(&self) -> usize {
        self.repeat.unwrap_or(1) * self.pending_repeat.unwrap_or(1)
    }
}
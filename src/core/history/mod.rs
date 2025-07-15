use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use crate::core::history::edit::Edit;

pub mod edit;

#[derive(Debug, Clone, Default)]
pub struct History {
    edits: VecDeque<Edit>,
    redos: VecDeque<Edit>,
    max_size: usize,
    last_action_time: Option<std::time::Instant>,
    group_timeout: Duration,
}

impl History {
    pub fn new(size: usize) -> Self {
        Self {
            edits: VecDeque::with_capacity(size),
            redos: VecDeque::with_capacity(size),
            max_size: size,
            last_action_time: None,
            group_timeout: Duration::from_millis(500),
        }
    }

    pub fn push(&mut self, change: Edit) {
        self.redos.clear();

        let now = Instant::now();

        // Check if we are still in the same action group
        let should_group = self.last_action_time.map_or(false, |last_time| {
            now.duration_since(last_time) <= self.group_timeout
        });

        if should_group {
            if let Some(last_change) = self.edits.pop_back() {
                if let Some(merged) = last_change.merge(&change) {
                    self.edits.push_back(merged);
                } else {
                    self.edits.push_back(last_change);
                    self.edits.push_back(change);
                }
            }
        } else {
            self.edits.push_back(change);
        }

        self.last_action_time = Some(now);
        // Ensure we don't exceed max size
        while self.edits.len() > self.max_size {
            self.edits.pop_front();
        }
    }

    pub fn undo(&mut self) -> Option<Edit> {
        if let Some(change) = self.edits.pop_back() {
            let undo = change.undo();
            self.redos.push_back(change);
            Some(undo)
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<Edit> {
        if let Some(change) = self.redos.pop_back() {
            self.edits.push_back(change.clone());
            Some(change)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.edits.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redos.is_empty()
    }

    pub fn clear(&mut self) {
        self.edits.clear();
        self.redos.clear();
        self.last_action_time = None;
    }

    pub fn break_group(&mut self) {
        self.last_action_time = Some(Instant::now());
    }
}

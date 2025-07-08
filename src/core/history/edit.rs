use tree_sitter::{InputEdit, Point};

#[derive(Debug, Clone)]
pub struct Transition {
    pub before: Point,
    pub after: Point,
}

impl Transition {
    pub fn new(before: Point, after: Point) -> Self {
        Self { before, after }
    }
}

#[derive(Debug, Clone)]
pub enum Edit {
    Insert(Insert),
    Delete(Delete),
}

impl Edit {
    pub fn insert(
        start_byte: usize,
        start_point: Point,
        text: String,
        start: Point,
        end: Point,
    ) -> Self {
        Edit::Insert(Insert::new(
            start_byte,
            start_point,
            text,
            Transition::new(start, end),
        ))
    }

    pub fn delete(
        start_byte: usize,
        start_point: Point,
        text: String,
        start: Point,
        end: Point,
    ) -> Self {
        Edit::Delete(Delete::new(
            start_byte,
            start_point,
            text,
            Transition::new(start, end),
        ))
    }

    pub fn merge(&self, other: &Edit) -> Option<Edit> {
        match (self, other) {
            (Edit::Insert(i1), Edit::Insert(i2)) => i1.merge(i2).map(Edit::Insert),
            (Edit::Delete(d1), Edit::Delete(d2)) => d1.merge(d2).map(Edit::Delete),
            _ => None, // Only allow merging of same types
        }
    }

    pub fn undo(&self) -> Edit {
        match self {
            Edit::Insert(Insert {
                start_byte: position,
                start_point,
                text,
                transition: point,
            }) => Edit::delete(
                *position,
                *start_point,
                text.clone(),
                point.after,
                point.before,
            ),
            Edit::Delete(Delete {
                start_byte: position,
                start_point,
                text,
                transition: point,
            }) => Edit::insert(
                *position,
                *start_point,
                text.clone(),
                point.after,
                point.before,
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct Insert {
    pub start_byte: usize,
    pub start_point: Point,
    pub text: String,
    pub transition: Transition,
}

impl Insert {
    pub fn new(
        start_byte: usize,
        start_point: Point,
        text: String,
        transition: Transition,
    ) -> Self {
        Self {
            start_byte,
            start_point,
            text,
            transition,
        }
    }

    pub fn merge(&self, other: &Self) -> Option<Insert> {
        // Only merge single characters (don't merge paste operations)
        let self_count = self.text.chars().count();
        let other_count = other.text.chars().count();
        if self_count == 0 || other_count == 0 || other_count > 1 {
            return None;
        }

        let char1 = self.text.chars().last().unwrap();
        let char2 = other.text.chars().next().unwrap();

        // Check if characters can be grouped
        if !chars_can_group(char1, char2) {
            return None;
        }

        // Must be consecutive positions
        if other.start_byte != self.start_byte + self.text.len() {
            return None;
        }

        // Cursor positions must connect properly
        if self.transition.after != other.transition.before {
            return None;
        }

        // Create merged insert
        Some(Insert {
            start_byte: self.start_byte,
            start_point: self.start_point,
            text: format!("{}{}", self.text, other.text),
            transition: Transition::new(self.transition.before, other.transition.after),
        })
    }

    pub fn edit_summary(&self) -> InputEdit {
        InputEdit {
            start_byte: self.start_byte,
            old_end_byte: self.start_byte,
            new_end_byte: self.start_byte + self.text.len(),
            start_position: self.start_point,
            old_end_position: self.start_point,
            new_end_position: get_end_position(&self.text, &self.start_point),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Delete {
    pub start_byte: usize,
    pub start_point: Point,
    pub text: String,
    pub transition: Transition,
}

impl Delete {
    pub fn new(
        start_byte: usize,
        start_point: Point,
        text: String,
        transition: Transition,
    ) -> Self {
        Self {
            start_byte,
            start_point,
            text,
            transition,
        }
    }

    pub fn merge(&self, other: &Self) -> Option<Delete> {
        let self_count = self.text.chars().count();
        let other_count = other.text.chars().count();
        if self_count == 0 || other_count == 0 || other_count > 1 {
            return None;
        }

        let char1 = self.text.chars().last().unwrap();
        let char2 = other.text.chars().next().unwrap();

        // Check if characters can be grouped
        if !chars_can_group(char1, char2) {
            return None;
        }

        // 1. Backspace: delete backwards
        if other.start_byte + other.text.len() == self.start_byte
            && self.transition.after == other.transition.before
        {
            return Some(Delete {
                start_byte: other.start_byte,
                start_point: other.start_point,
                text: format!("{}{}", other.text, self.text),
                transition: Transition::new(self.transition.before, other.transition.after),
            });
        }

        // 2. Delete forward
        if self.start_byte == other.start_byte && self.transition.before == other.transition.before
        {
            // Create merged delete
            return Some(Delete {
                start_byte: self.start_byte,
                start_point: self.start_point,
                text: format!("{}{}", self.text, other.text),
                transition: Transition::new(self.transition.before, other.transition.after),
            });
        }
        None
    }

    pub fn edit_summary(&self) -> InputEdit {
        InputEdit {
            start_byte: self.start_byte,
            old_end_byte: self.start_byte + self.text.len(),
            new_end_byte: self.start_byte,
            start_position: self.start_point,
            old_end_position: get_end_position(&self.text, &self.start_point),
            new_end_position: self.start_point,
        }
    }
}

impl Edit {
    pub fn point_before(&self) -> Point {
        match self {
            Edit::Insert(insert) => insert.transition.before,
            Edit::Delete(delete) => delete.transition.before,
        }
    }

    pub fn point_after(&self) -> Point {
        match self {
            Edit::Insert(insert) => insert.transition.after,
            Edit::Delete(delete) => delete.transition.after,
        }
    }
}

fn chars_can_group(c1: char, c2: char) -> bool {
    if c1.is_alphanumeric() && c2.is_alphanumeric() {
        return true;
    }
    if c1.is_whitespace() && c2.is_whitespace() {
        return true;
    }
    if c1.is_ascii_punctuation() && c2.is_ascii_punctuation() {
        return true;
    }
    false
}

fn get_end_position(text: &str, start: &Point) -> Point {
    let mut end_position = start.clone();
    for b in text.as_bytes() {
        if *b == b'\n' {
            end_position.row += 1;
            end_position.column = 0;
        } else {
            end_position.column += 1;
        }
    }
    end_position
}

use tree_sitter::Point;

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
pub enum Change {
    Insert(Insert),
    Delete(Delete),
    Multiple {
        changes: Vec<Change>,
        point: Transition,
    },
}

impl Change {
    pub fn insert(position: usize, text: String, start: Point, end: Point) -> Self {
        Change::Insert(Insert::new(position, text, Transition::new(start, end)))
    }

    pub fn delete(position: usize, text: String, start: Point, end: Point) -> Self {
        Change::Delete(Delete::new(position, text, Transition::new(start, end)))
    }

    pub fn multiple(changes: Vec<Change>, start: Point, end: Point) -> Self {
        Change::Multiple {
            changes,
            point: Transition::new(start, end),
        }
    }

    pub fn merge(&self, other: &Change) -> Option<Change> {
        match (self, other) {
            (Change::Insert(i1), Change::Insert(i2)) => i1.merge(i2).map(Change::Insert),
            (Change::Delete(d1), Change::Delete(d2)) => d1.merge(d2).map(Change::Delete),
            _ => None, // Only allow merging of same types
        }
    }

    pub fn undo(&self) -> Change {
        match self {
            Change::Insert(Insert {
                byte_position: position,
                text,
                point,
            }) => Change::delete(*position, text.clone(), point.after, point.before),
            Change::Delete(Delete {
                byte_position: position,
                text,
                point,
            }) => Change::insert(*position, text.clone(), point.after, point.before),
            Change::Multiple { changes, point } => {
                let changes = changes.iter().map(Self::undo).rev().collect();
                Change::multiple(changes, point.after, point.before)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Insert {
    pub byte_position: usize,
    pub text: String,
    pub point: Transition,
}

impl Insert {
    pub fn new(position: usize, text: String, point: Transition) -> Self {
        Self {
            byte_position: position,
            text,
            point,
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
        if other.byte_position != self.byte_position + self.text.len() {
            return None;
        }

        // Cursor positions must connect properly
        if self.point.after != other.point.before {
            return None;
        }

        // Create merged insert
        Some(Insert {
            byte_position: self.byte_position,
            text: format!("{}{}", self.text, other.text),
            point: Transition::new(self.point.before, other.point.after),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Delete {
    pub byte_position: usize,
    pub text: String,
    pub point: Transition,
}

impl Delete {
    pub fn new(position: usize, text: String, point: Transition) -> Self {
        Self {
            byte_position: position,
            text,
            point,
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
        if other.byte_position + other.text.len() == self.byte_position
            && self.point.after == other.point.before
        {
            return Some(Delete {
                byte_position: other.byte_position,
                text: format!("{}{}", other.text, self.text),
                point: Transition::new(self.point.before, other.point.after),
            });
        }

        // 2. Delete forward
        if self.byte_position == other.byte_position && self.point.before == other.point.before {
            // Create merged delete
            return Some(Delete {
                byte_position: self.byte_position,
                text: format!("{}{}", self.text, other.text),
                point: Transition::new(self.point.before, other.point.after),
            });
        }
        None
    }
}

impl Change {
    pub fn point_before(&self) -> Point {
        match self {
            Change::Insert(insert) => insert.point.before,
            Change::Delete(delete) => delete.point.before,
            Change::Multiple { point, .. } => point.before,
        }
    }

    pub fn point_after(&self) -> Point {
        match self {
            Change::Insert(insert) => insert.point.after,
            Change::Delete(delete) => delete.point.after,
            Change::Multiple { point, .. } => point.after,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_merge() {
        let insert1 = Insert::new(
            0,
            "a".into(),
            Transition::new(Point::new(0, 0), Point::new(0, 1)),
        );
        let insert2 = Insert::new(
            1,
            "b".into(),
            Transition::new(Point::new(0, 1), Point::new(0, 2)),
        );
        let merged = insert1.merge(&insert2);
        assert!(merged.is_some());
        let merged = merged.unwrap();
        assert_eq!(merged.byte_position, 0);
        assert_eq!(merged.text, "ab");
        assert_eq!(merged.point.before, Point::new(0, 0));
        assert_eq!(merged.point.after, Point::new(0, 2));
    }

    #[test]
    fn test_delete_merge_forward() {
        let delete1 = Delete::new(
            0,
            "a".into(),
            Transition::new(Point::new(0, 0), Point::new(0, 0)),
        );
        let delete2 = Delete::new(
            0,
            "b".into(),
            Transition::new(Point::new(0, 0), Point::new(0, 0)),
        );
        let merged = delete1.merge(&delete2);
        assert!(merged.is_some());
        let merged = merged.unwrap();
        assert_eq!(merged.byte_position, 0);
        assert_eq!(merged.text, "ab");
        assert_eq!(merged.point.before, Point::new(0, 0));
        assert_eq!(merged.point.after, Point::new(0, 0));
    }

    #[test]
    fn test_delete_merge_backward() {
        let delete1 = Delete::new(
            4,
            "b".into(),
            Transition::new(Point::new(0, 4), Point::new(0, 3)),
        );
        let delete2 = Delete::new(
            3,
            "a".into(),
            Transition::new(Point::new(0, 3), Point::new(0, 2)),
        );
        let merged = delete1.merge(&delete2);
        assert!(merged.is_some());
        let merged = merged.unwrap();
        assert_eq!(merged.byte_position, 0);
        assert_eq!(merged.text, "pu");
        assert_eq!(merged.point.before, Point::new(0, 2));
        assert_eq!(merged.point.after, Point::new(0, 0));
    }
}

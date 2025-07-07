pub struct Utf8CharIterator<'a> {
    bytes: &'a [u8],
    byte_pos: usize,
    char_pos: usize,
}

#[derive(Debug, Clone)]
pub struct CharPosition {
    pub character: char,
    pub char_index: usize,
    pub byte_index: usize,
    pub byte_len: usize, // Length of this character in bytes
}

impl<'a> Utf8CharIterator<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            byte_pos: 0,
            char_pos: 0,
        }
    }

    pub fn from_byte_pos(bytes: &'a [u8], start_byte_pos: usize) -> Self {
        let mut iter = Self::new(bytes);
        
        // Fast-forward to the starting position
        while iter.byte_pos < start_byte_pos && iter.byte_pos < bytes.len() {
            if iter.next().is_none() {
                break;
            }
        }
        
        iter
    }

    pub fn from_char_pos(bytes: &'a [u8], start_char_pos: usize) -> Self {
        let mut iter = Self::new(bytes);
        
        // Fast-forward to the starting position
        while iter.char_pos < start_char_pos {
            if iter.next().is_none() {
                break;
            }
        }
        
        iter
    }

    pub fn byte_pos_for_char_pos(bytes: &'a [u8], target_char_pos: usize) -> Option<usize> {
        let mut iter = Self::new(bytes);
        
        while let Some(char_pos) = iter.next() {
            if char_pos.char_index >= target_char_pos {
                return Some(char_pos.byte_index);
            }
        }
        
        None
    }

    pub fn char_pos_for_byte_pos(bytes: &'a [u8], target_byte_pos: usize) -> Option<usize> {
        let mut iter = Self::new(bytes);
        
        while let Some(char_pos) = iter.next() {
            if char_pos.byte_index >= target_byte_pos {
                return Some(char_pos.char_index);
            }
        }
        
        None
    }
}

impl<'a> Iterator for Utf8CharIterator<'a> {
    type Item = CharPosition;

    fn next(&mut self) -> Option<Self::Item> {
        if self.byte_pos >= self.bytes.len() {
            return None;
        }

        let start_byte = self.byte_pos;
        let first_byte = self.bytes[self.byte_pos];
        
        let (char_len, char_value) = if first_byte < 0x80 {
            // ASCII character
            (1, first_byte as char)
        } else if first_byte < 0xE0 {
            // 2-byte UTF8
            if self.byte_pos + 2 >= self.bytes.len() {
                return None;
            }
            let bytes = [first_byte, self.bytes[self.byte_pos + 1]];
            match std::str::from_utf8(&bytes) {
                Ok(s) => (2, s.chars().next().unwrap_or('\u{FFFD}')),
                Err(_) => (1, '\u{FFFD}'), // Replacement character for invalid UTF-8
            }
        } else if first_byte < 0xF0 {
            // 3-byte UTF-8
            if self.byte_pos + 3 >= self.bytes.len() {
                return None;
            }
            let bytes = [
                first_byte,
                self.bytes[self.byte_pos + 1],
                self.bytes[self.byte_pos + 2],
            ];
            match std::str::from_utf8(&bytes) {
                Ok(s) => (3, s.chars().next().unwrap_or('\u{FFFD}')),
                Err(_) => (1, '\u{FFFD}'),
            }
        } else {
            // 4-byte UTF-8
            if self.byte_pos + 3 >= self.bytes.len() {
                return None;
            }
            let bytes = [
                first_byte,
                self.bytes[self.byte_pos + 1],
                self.bytes[self.byte_pos + 2],
                self.bytes[self.byte_pos + 3],
            ];
            match std::str::from_utf8(&bytes) {
                Ok(s) => (4, s.chars().next().unwrap_or('\u{FFFD}')),
                Err(_) => (1, '\u{FFFD}'),
            }
        };
        
        let position = CharPosition {
            character: char_value,
            char_index: self.char_pos,
            byte_index: start_byte,
            byte_len: char_len,
        };
        
        self.byte_pos += char_len;
        self.char_pos += 1;

        Some(position)
    }
}

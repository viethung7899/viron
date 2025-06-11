#[derive(Debug)]
pub struct GapBuffer<T> {
    pub(super) buffer: Vec<T>,
    pub(super) gap_start: usize,
    pub(super) gap_end: usize,
}

const INITIAL_CAPACITY: usize = 1024;

impl<T: Default + Clone> Default for GapBuffer<T> {
    fn default() -> Self {
        Self {
            buffer: vec![T::default(); INITIAL_CAPACITY],
            gap_start: 0,
            gap_end: INITIAL_CAPACITY,
        }
    }
}

impl<T: Clone + Copy + Default> GapBuffer<T> {
    pub fn from_slice(slice: &[T]) -> Self {
        let length = slice.len();
        let capacity = length + INITIAL_CAPACITY;
        let mut buffer = vec![T::default(); capacity];
        buffer[..length].copy_from_slice(slice);
        Self {
            buffer,
            gap_start: length,
            gap_end: capacity,
        }
    }

    pub fn prefix_len(&self) -> usize {
        self.gap_start
    }

    pub fn suffix_len(&self) -> usize {
        self.buffer.len() - self.gap_end
    }

    pub fn len_without_gap(&self) -> usize {
        self.buffer.len() - self.gap_len()
    }

    pub fn gap_len(&self) -> usize {
        self.gap_end - self.gap_start
    }

    pub fn expand_gap(&mut self, new_capacity: usize) {
        let mut new_buffer = vec![T::default(); new_capacity];

        let prefix_len = self.gap_start;
        let suffix_len = self.buffer.len() - self.gap_end;

        new_buffer[..prefix_len].copy_from_slice(&self.buffer[..prefix_len]);
        new_buffer[new_capacity - suffix_len..].copy_from_slice(&self.buffer[self.gap_end..]);

        self.gap_end = new_capacity - suffix_len;
        self.buffer = new_buffer;
    }

    pub fn move_gap(&mut self, position: usize) {
        if position < self.gap_start {
            let distance = self.gap_start - position;
            self.buffer
                .copy_within(position..self.gap_start, self.gap_end - distance);
            self.gap_start = position;
            self.gap_end -= distance;
        } else if position > self.gap_start {
            let distance = position - self.gap_start;
            self.buffer
                .copy_within(self.gap_end..self.gap_end + distance, self.gap_start);
            self.gap_start += distance;
            self.gap_end += distance;
        }
    }

    pub fn insert_single(&mut self, value: T) {
        if self.gap_len() == 0 {
            self.expand_gap(2 * self.buffer.len());
        }
        self.buffer[self.gap_start] = value;
        self.gap_start += 1;
    }

    pub fn insert_multiple(&mut self, values: &[T]) {
        if self.gap_len() < values.len() {
            let mut capacity = self.buffer.len();
            while capacity < values.len() {
                capacity *= 2;
            }
            self.expand_gap(capacity);
        }
        self.buffer[self.gap_start..self.gap_start + values.len()].copy_from_slice(values);
        self.gap_start += values.len();
    }

    pub fn remove_single(&mut self) -> Option<T> {
        let value = self.buffer.get(self.gap_end);
        if value.is_none() {
            self.gap_end += 1;
        }
        value.copied()
    }

    pub fn delete(&mut self, count: usize) -> Option<Vec<T>> {
        let values = self.buffer.get(self.gap_end..self.gap_end + count);
        if values.is_none() {
            self.gap_end += count;
        }
        values.as_ref().map(|v| v.to_vec())
    }
}

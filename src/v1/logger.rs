use std::{fs::File, io::Write, sync::Mutex};

pub struct Logger {
    file: Mutex<File>,
}

impl Logger {
    pub fn new(file_path: &str) -> Self {
        let file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(file_path)
            .expect("Failed to open log file");

        Self {
            file: Mutex::new(file),
        }
    }

    pub fn log(&self, message: &str) {
        let mut file = self.file.lock().unwrap();
        writeln!(file, "{}", message).unwrap();
    }
}

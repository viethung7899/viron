use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct VersionedContents {
    contents: HashMap<String, String>,
    versions: HashMap<String, i32>,
}

impl VersionedContents {
    pub fn update_document(&mut self, path: &str, content: String) {
        self.contents.insert(path.to_string(), content);
        let version = self.versions.entry(path.to_string()).or_insert(0);
        *version += 1;
    }

    pub fn get_version(&self, path: &str) -> i32 {
        self.versions.get(path).cloned().unwrap_or_default()
    }

    pub fn get_content(&self, path: &str) -> &str {
        self.contents.get(path).map(String::as_str).unwrap_or("")
    }
}

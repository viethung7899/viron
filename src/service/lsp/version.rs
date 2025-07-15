use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct VersionedContents {
    contents: HashMap<String, String>,
    versions: HashMap<String, i32>,
}

impl VersionedContents {
    pub fn update_document(&mut self, uri: &str, content: String) {
        self.contents.insert(uri.to_string(), content);
        let version = self.versions.entry(uri.to_string()).or_insert(0);
        *version += 1;
    }

    pub fn get_version(&self, uri: &str) -> i32 {
        self.versions.get(uri).cloned().unwrap_or_default()
    }

    pub fn get_content(&self, uri: &str) -> &str {
        self.contents.get(uri).map(String::as_str).unwrap_or("")
    }
}

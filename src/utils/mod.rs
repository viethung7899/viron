use std::path::PathBuf;

pub fn absolutize(path: &str) -> anyhow::Result<PathBuf> {
    let current = std::env::current_dir()?;
    let full_path = std::fs::canonicalize(current.join(path))?;
    Ok(full_path)
}

//! Cross-platform file and directory operations.

use crate::{Error, Result};
use std::path::{Path, PathBuf};

fn io<T>(operation: &'static str, result: std::io::Result<T>) -> Result<T> {
    result.map_err(|error| Error::new(operation, error))
}

pub fn read_text(path: impl AsRef<Path>) -> Result<String> {
    io("filesystem.read_text", std::fs::read_to_string(path))
}

pub fn write_text(path: impl AsRef<Path>, text: &str) -> Result<()> {
    io("filesystem.write_text", std::fs::write(path, text))
}

pub fn append_text(path: impl AsRef<Path>, text: &str) -> Result<()> {
    use std::io::Write;
    let mut file = io(
        "filesystem.append_text",
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path),
    )?;
    io("filesystem.append_text", file.write_all(text.as_bytes()))
}

pub fn create_directory(path: impl AsRef<Path>) -> Result<()> {
    io("filesystem.create_directory", std::fs::create_dir_all(path))
}

pub fn remove(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if path.is_dir() {
        io("filesystem.remove", std::fs::remove_dir_all(path))
    } else {
        io("filesystem.remove", std::fs::remove_file(path))
    }
}

pub fn exists(path: impl AsRef<Path>) -> bool {
    path.as_ref().exists()
}

pub fn list(path: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let mut entries = io("filesystem.list", std::fs::read_dir(path))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()
        .map_err(|error| Error::new("filesystem.list", error))?;
    entries.sort();
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_file_lifecycle() {
        let root = std::env::temp_dir().join(format!("adamantium-stdlib-{}", std::process::id()));
        let file = root.join("value.txt");
        let _ = std::fs::remove_dir_all(&root);
        create_directory(&root).unwrap();
        write_text(&file, "one").unwrap();
        append_text(&file, " two").unwrap();
        assert_eq!(read_text(&file).unwrap(), "one two");
        assert_eq!(list(&root).unwrap(), [file]);
        remove(&root).unwrap();
        assert!(!exists(&root));
    }
}

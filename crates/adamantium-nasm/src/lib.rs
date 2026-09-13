use std::path::Path;

pub fn object_path(source: &Path, windows: bool) -> std::path::PathBuf {
    source.with_extension(if windows { "obj" } else { "o" })
}

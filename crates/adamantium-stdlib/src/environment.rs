//! Read-only access to the host environment.

pub fn get(name: &str) -> Option<String> {
    std::env::var_os(name).map(|value| value.to_string_lossy().into_owned())
}

pub fn exists(name: &str) -> bool {
    std::env::var_os(name).is_some()
}

pub fn current_directory() -> crate::Result<String> {
    std::env::current_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| crate::Error::new("environment.current_directory", error))
}

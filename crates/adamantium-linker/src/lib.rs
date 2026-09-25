use std::path::{Path, PathBuf};

pub fn executable_name(name: &str, windows: bool) -> String {
    if windows {
        format!("{name}.exe")
    } else {
        name.to_owned()
    }
}

pub fn select_windows_linker(
    configured: Option<PathBuf>,
    tools: Option<&Path>,
    rust_sysroot: Option<&Path>,
) -> Option<PathBuf> {
    if let Some(configured) = configured {
        return Some(configured);
    }
    let bundled = tools.map(|directory| directory.join("lld-link.exe"));
    if bundled.as_ref().is_some_and(|path| path.is_file()) {
        return bundled;
    }
    let rust_lld =
        rust_sysroot.map(|root| root.join("lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe"));
    rust_lld.filter(|path| path.is_file())
}

pub fn windows_linker_driver_arguments(linker: &Path) -> &'static [&'static str] {
    let name = linker
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if name.eq_ignore_ascii_case("rust-lld") || name.eq_ignore_ascii_case("lld") {
        &["-flavor", "link"]
    } else {
        &[]
    }
}

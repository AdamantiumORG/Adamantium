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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arm64Platform {
    Windows,
    Linux,
}

pub fn arm64_link_arguments(
    platform: Arm64Platform,
    object: &Path,
    runtime: &Path,
    output: &Path,
) -> Vec<String> {
    match platform {
        Arm64Platform::Windows => vec![
            "-flavor".into(),
            "link".into(),
            "/machine:arm64".into(),
            "/subsystem:console".into(),
            "/entry:mainCRTStartup".into(),
            format!("/out:{}", output.display()),
            object.display().to_string(),
            runtime.display().to_string(),
        ],
        Arm64Platform::Linux => vec![
            "-target".into(),
            "aarch64-unknown-linux-gnu".into(),
            object.display().to_string(),
            runtime.display().to_string(),
            "-o".into(),
            output.display().to_string(),
        ],
    }
}

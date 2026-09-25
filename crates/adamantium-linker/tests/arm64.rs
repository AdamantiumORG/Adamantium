use adamantium_linker::{Arm64Platform, arm64_link_arguments};
use std::path::Path;

#[test]
fn selects_arm64_for_windows_and_linux_linkers() {
    let windows = arm64_link_arguments(
        Arm64Platform::Windows,
        Path::new("main.obj"),
        Path::new("runtime.lib"),
        Path::new("main.exe"),
    );
    assert!(windows.iter().any(|argument| argument == "/machine:arm64"));
    let linux = arm64_link_arguments(
        Arm64Platform::Linux,
        Path::new("main.o"),
        Path::new("runtime.a"),
        Path::new("main"),
    );
    assert!(
        linux
            .iter()
            .any(|argument| argument == "aarch64-unknown-linux-gnu")
    );
}

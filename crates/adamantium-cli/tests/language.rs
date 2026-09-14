#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::process::Command;

#[test]
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn language_conformance_suite() {
    let output = Command::new(env!("CARGO_BIN_EXE_adamantium"))
        .args([
            "test",
            "language",
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests"),
            "--verbose",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

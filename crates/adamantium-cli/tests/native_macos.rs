#![cfg(target_os = "macos")]

use std::{fs, process::Command};

#[test]
fn builds_and_runs_macos_x86_64_executable() {
    assert_eq!(std::env::consts::ARCH, "x86_64");
    let root = std::env::temp_dir().join(format!("adamantium-macos-native-{}", std::process::id()));
    fs::create_dir_all(root.join("code")).unwrap();
    fs::write(
        root.join("project.toml"),
        "name=\"MacNative\"\nversion=\"1.0.0\"\ndescription=\"\"\nauthors=[]\n",
    )
    .unwrap();
    fs::write(root.join("requirement.toml"), "[packages]\n").unwrap();
    fs::write(
        root.join("code/main.ad"),
        "fun main(value:int) { var result=value*2; print.newline(result); }",
    )
    .unwrap();

    let build = Command::new(env!("CARGO_BIN_EXE_adamantium"))
        .args(["build", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&build.stdout),
        String::from_utf8_lossy(&build.stderr)
    );

    let executable = root.join("target/MacNative");
    let output = Command::new(&executable)
        .args(["--value", "21"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "42");

    fs::write(root.join("code/main.ad"), "fun main() { exit(code=23); }").unwrap();
    let build = Command::new(env!("CARGO_BIN_EXE_adamantium"))
        .args(["build", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let output = Command::new(executable).output().unwrap();
    assert_eq!(output.status.code(), Some(23));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    fs::remove_dir_all(root).unwrap();
}

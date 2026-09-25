#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fs, process::Command};

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

#[test]
#[cfg(any(target_os = "windows", target_os = "linux"))]
fn optimization_levels_preserve_program_output() {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "adamantium-optimization-levels-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let created = Command::new(env!("CARGO_BIN_EXE_adamantium"))
        .args(["new", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(created.status.success());
    fs::write(root.join("code/main.ad"), "fun helper(a:int) r:int { r=a+0; } fun deadfunc() r:None { print.newline(99); } fun main() { var a=2+3; print.newline(helper(a)); }").unwrap();
    let name = root.file_name().unwrap().to_string_lossy();
    let mut outputs = Vec::new();
    for level in ["-O0", "-O1", "-O2"] {
        let build = Command::new(env!("CARGO_BIN_EXE_adamantium"))
            .args(["build", root.to_str().unwrap(), level])
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{level}: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        let assembly = fs::read_to_string(root.join("target").join(format!("{name}.asm"))).unwrap();
        assert_eq!(assembly.contains("ad_fun_deadfunc:"), level != "-O2");
        let executable = root.join("target").join(if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_string()
        });
        let output = Command::new(executable)
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(output.status.success());
        outputs.push(output.stdout);
    }
    assert!(outputs.windows(2).all(|pair| pair[0] == pair[1]));
    assert_eq!(String::from_utf8_lossy(&outputs[0]).trim(), "5");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn published_examples_pass_static_analysis() {
    let examples = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut directories = fs::read_dir(&examples)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    directories.sort();
    assert!(!directories.is_empty(), "no published examples found");

    for directory in directories {
        let output = Command::new(env!("CARGO_BIN_EXE_adamantium"))
            .arg("check")
            .arg(&directory)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}:\n{}",
            directory.display(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

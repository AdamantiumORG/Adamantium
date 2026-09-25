#[test]
fn appends_windows_extension() {
    assert_eq!(adamantium_linker::executable_name("app", true), "app.exe");
}

#[test]
fn windows_linker_prefers_configuration_then_bundle_then_rust_lld() {
    let root = std::env::temp_dir().join(format!("adamantium-linker-{}", std::process::id()));
    let tools = root.join("tools");
    let sysroot = root.join("rust");
    let rust_lld = sysroot.join("lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe");
    std::fs::create_dir_all(rust_lld.parent().unwrap()).unwrap();
    std::fs::create_dir_all(&tools).unwrap();
    std::fs::write(&rust_lld, []).unwrap();

    assert_eq!(
        adamantium_linker::select_windows_linker(None, Some(&tools), Some(&sysroot)),
        Some(rust_lld.clone())
    );
    let bundled = tools.join("lld-link.exe");
    std::fs::write(&bundled, []).unwrap();
    assert_eq!(
        adamantium_linker::select_windows_linker(None, Some(&tools), Some(&sysroot)),
        Some(bundled)
    );
    let configured = root.join("custom-linker.exe");
    assert_eq!(
        adamantium_linker::select_windows_linker(
            Some(configured.clone()),
            Some(&tools),
            Some(&sysroot)
        ),
        Some(configured)
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn selects_the_windows_flavor_for_generic_lld_drivers() {
    use std::path::Path;

    assert_eq!(
        adamantium_linker::windows_linker_driver_arguments(Path::new("rust-lld.exe")),
        ["-flavor", "link"]
    );
    assert_eq!(
        adamantium_linker::windows_linker_driver_arguments(Path::new("lld.exe")),
        ["-flavor", "link"]
    );
    assert!(
        adamantium_linker::windows_linker_driver_arguments(Path::new("lld-link.exe")).is_empty()
    );
}

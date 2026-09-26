use adamantium_stdlib::{environment, filesystem, process};

#[test]
fn paths_with_unicode_round_trip_as_text() {
    let root = std::env::temp_dir().join(format!(
        "adamantium-cross-platform-{}-\u{105}",
        std::process::id()
    ));
    let file = root.join("warto\u{15b}\u{107}.txt");
    let _ = std::fs::remove_dir_all(&root);
    filesystem::create_directory(&root).unwrap();
    filesystem::write_text(&file, "\u{17c}\u{f3}\u{142}w\n").unwrap();
    assert_eq!(
        filesystem::read_text(&file).unwrap(),
        "\u{17c}\u{f3}\u{142}w\n"
    );
    assert_eq!(filesystem::list(&root).unwrap(), [file]);
    filesystem::remove(&root).unwrap();
}

#[test]
fn process_arguments_are_not_interpreted_by_a_shell() {
    #[cfg(windows)]
    let output = process::Command::new("cmd")
        .argument("/d")
        .argument("/c")
        .argument("echo")
        .argument("Adamantium")
        .run()
        .unwrap();

    #[cfg(unix)]
    let output = process::Command::new("printf")
        .argument("%s")
        .argument("Adamantium")
        .run()
        .unwrap();

    assert_eq!(output.exit_code, Some(0));
    assert!(output.stdout.contains("Adamantium"));
    assert!(output.stderr.is_empty());
}

#[test]
fn current_directory_is_available_on_every_supported_host() {
    let directory = environment::current_directory().unwrap();
    assert!(!directory.is_empty());
    assert!(std::path::Path::new(&directory).is_dir());
}

#[test]
fn appends_windows_extension() {
    assert_eq!(adamantium_linker::executable_name("app", true), "app.exe");
}

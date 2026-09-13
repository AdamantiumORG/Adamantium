#[test]
fn selects_platform_object_extension() {
    assert_eq!(
        adamantium_nasm::object_path(std::path::Path::new("main.asm"), true)
            .extension()
            .unwrap(),
        "obj"
    );
}

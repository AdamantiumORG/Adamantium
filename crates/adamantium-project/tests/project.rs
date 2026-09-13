#[test]
fn resolves_main_source() {
    assert!(
        adamantium_project::ProjectLayout::new("demo")
            .source()
            .ends_with("demo/code/main.ad")
    );
}

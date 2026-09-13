#[test]
fn connects_compiler_layers() {
    assert!(
        adamantium_compiler::architecture_smoke_test("fun main")
            .unwrap()
            .contains("ret")
    );
}

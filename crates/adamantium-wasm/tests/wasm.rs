#[test]
fn recognizes_wasm_magic() {
    assert!(adamantium_wasm::is_module(b"\0asm\x01\0\0\0"));
}

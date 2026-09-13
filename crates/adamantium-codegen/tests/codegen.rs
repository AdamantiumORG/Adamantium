#[test]
fn emits_nasm_for_constant() {
    assert_eq!(
        adamantium_codegen::emit(&[adamantium_ir::integer(7)]),
        "mov rax, 7\n"
    );
}

#[test]
fn creates_typed_integer_instruction() {
    assert!(matches!(
        adamantium_ir::integer(4),
        adamantium_ir::Instruction::Constant { value: 4, .. }
    ));
}

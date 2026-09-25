#[test]
fn infers_default_integer() {
    assert_eq!(
        adamantium_types::infer_literal("10"),
        adamantium_types::PrimitiveType::Int
    );
}

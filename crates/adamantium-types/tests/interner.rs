use adamantium_types::{Type, TypeInterner, TypeKind};

#[test]
fn canonical_types_receive_stable_ids() {
    let mut types = TypeInterner::default();
    let integer = types.intern(TypeKind::Primitive(Type::I32));
    assert_eq!(integer, types.intern(TypeKind::Primitive(Type::I32)));
    let list = types.intern(TypeKind::List(integer));
    assert_eq!(types.kind(list), Some(&TypeKind::List(integer)));
}

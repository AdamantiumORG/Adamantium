#[test]
fn runtime_exports_a_versioned_abi() {
    assert_eq!(std::mem::size_of::<adamantium_runtime::types::Value>(), 16);
}

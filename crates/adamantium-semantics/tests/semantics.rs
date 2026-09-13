#[test]
fn reports_duplicate_identifiers() {
    assert_eq!(adamantium_semantics::analyze("name name").len(), 1);
}

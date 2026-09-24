#[test]
fn reports_duplicate_identifiers() {
    let tokens = adamantium_lexer::lex("name name").unwrap();
    assert_eq!(adamantium_semantics::analyze("name name", &tokens).len(), 1);
}

#[test]
fn reports_duplicate_identifiers() {
    let tokens = adamantium_lexer::lex("name name").unwrap();
    assert_eq!(adamantium_semantics::analyze(&tokens).len(), 1);
}

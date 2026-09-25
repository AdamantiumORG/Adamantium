#[test]
fn reports_duplicate_identifiers() {
    let tokens = adamantium_lexer::lex("name name").unwrap();
    assert_eq!(adamantium_semantics::analyze("name name", &tokens).len(), 1);
}

#[test]
fn name_resolution_and_type_checking_have_separate_outputs() {
    let source = "value";
    let tokens = adamantium_lexer::lex(source).unwrap();
    let parsed = adamantium_parser::parse(source, &tokens);
    let resolution = adamantium_semantics::resolve(&parsed).unwrap();
    assert_eq!(resolution.definitions.len(), 1);

    let mut types = adamantium_types::TypeInterner::default();
    let hir = adamantium_semantics::type_check(&resolution, &mut types);
    assert_eq!(hir.definitions[0].span, resolution.definitions[0].span);
    assert!(types.kind(hir.definitions[0].ty).is_some());
}

use adamantium_ast::{Identifier, Span};

#[test]
fn preserves_identifier_source_span() {
    let id = Identifier::new(
        "main",
        Span {
            start: 4,
            end: 8,
            line: 2,
            column: 5,
        },
    );
    assert_eq!(
        (id.name.as_str(), id.span.line, id.span.column),
        ("main", 2, 5)
    );
}

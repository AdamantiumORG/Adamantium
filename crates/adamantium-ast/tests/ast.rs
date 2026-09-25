use adamantium_ast::{BinaryOperator, Expression, Identifier, Span};

#[test]
fn preserves_identifier_source_span() {
    let id = Identifier::new("main", Span { start: 4, end: 8 });
    assert_eq!(
        (id.name.as_str(), id.span),
        ("main", Span { start: 4, end: 8 })
    );
}

#[test]
fn binary_expression_stores_semantic_operator_and_source_span() {
    let expression = Expression::Binary {
        operator: BinaryOperator::Add,
        left: Box::new(Expression::Identifier(Identifier::new(
            "left",
            Span { start: 0, end: 4 },
        ))),
        right: Box::new(Expression::Identifier(Identifier::new(
            "right",
            Span { start: 7, end: 12 },
        ))),
        span: Span { start: 0, end: 12 },
    };

    let Expression::Binary { operator, span, .. } = expression else {
        panic!("expected binary expression");
    };
    assert_eq!(operator, BinaryOperator::Add);
    assert_eq!(span, Span { start: 0, end: 12 });
}

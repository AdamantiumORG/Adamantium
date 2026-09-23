#[test]
fn parses_identifier_stream() {
    let tokens = adamantium_lexer::lex("fun main() {}").unwrap();
    let parsed = adamantium_parser::parse(&tokens);
    assert_eq!(parsed.identifiers[0].name, "main");
}

#[test]
fn parses_expression_from_token_kinds_with_precedence() {
    use adamantium_ast::{BinaryOperator, Expression};

    let tokens = adamantium_lexer::lex("a+b*2;").unwrap();
    let expression = adamantium_parser::parse_expression(&tokens).unwrap();
    let Expression::Binary {
        operator,
        left,
        right,
        span,
    } = expression
    else {
        panic!("expected binary expression");
    };
    assert_eq!(operator, BinaryOperator::Add);
    assert!(matches!(*left, Expression::Identifier(_)));
    assert!(matches!(
        *right,
        Expression::Binary {
            operator: BinaryOperator::Multiply,
            ..
        }
    ));
    assert_eq!((span.start, span.end), (0, 5));
}

#[test]
fn parser_reports_tokens_instead_of_retokenizing_text() {
    let tokens = adamantium_lexer::lex("a+b;").unwrap();
    assert!(adamantium_parser::parse_expression(&tokens).is_ok());

    let missing_operand = adamantium_lexer::lex("a+;").unwrap();
    let error = adamantium_parser::parse_expression(&missing_operand).unwrap_err();
    assert_eq!(error.span.start, 2);
    assert!(error.message.contains("expected"));
}

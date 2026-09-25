#[test]
fn parses_identifier_stream() {
    let tokens = adamantium_lexer::lex("fun main() {}").unwrap();
    let parsed = adamantium_parser::parse("fun main() {}", &tokens);
    assert_eq!(parsed.identifiers[0].name, "main");
}

#[test]
fn parses_expression_from_token_kinds_with_precedence() {
    use adamantium_ast::{BinaryOperator, Expression};

    let tokens = adamantium_lexer::lex("a+b*2;").unwrap();
    let expression = adamantium_parser::parse_expression("a+b*2;", &tokens).unwrap();
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
    assert!(adamantium_parser::parse_expression("a+b;", &tokens).is_ok());

    let missing_operand = adamantium_lexer::lex("a+;").unwrap();
    let error = adamantium_parser::parse_expression("a+;", &missing_operand).unwrap_err();
    assert_eq!(error.span.start, 2);
    assert!(error.message.contains("expected"));
}

#[test]
fn lexically_valid_missing_expression_is_a_parser_error() {
    let source = "var x = ;";
    let tokens = adamantium_lexer::lex(source).expect("the source is lexically valid");
    let errors = adamantium_parser::parse_checked(source, &tokens).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].message, "expected expression after '='");
    assert_eq!(tokens[3].kind, adamantium_lexer::TokenKind::Semicolon);
    assert_eq!(errors[0].span, tokens[3].span);
}

#[test]
fn invalid_characters_remain_lexer_errors() {
    let error = adamantium_lexer::lex("var x = @;").unwrap_err();
    assert!(matches!(
        error,
        adamantium_lexer::LexError::UnexpectedCharacter { character: '@', .. }
    ));
}

#[test]
fn parses_minus_as_unary_or_binary_from_context() {
    use adamantium_ast::{BinaryOperator, Expression, UnaryOperator};

    let tokens = adamantium_lexer::lex("a--b;").unwrap();
    let expression = adamantium_parser::parse_expression("a--b;", &tokens).unwrap();
    let Expression::Binary {
        operator,
        right,
        span,
        ..
    } = expression
    else {
        panic!("expected subtraction");
    };
    assert_eq!(operator, BinaryOperator::Subtract);
    assert!(matches!(
        *right,
        Expression::Unary {
            operator: UnaryOperator::Negate,
            ..
        }
    ));
    assert_eq!((span.start, span.end), (0, 4));

    let tokens = adamantium_lexer::lex("-a*2;").unwrap();
    let expression = adamantium_parser::parse_expression("-a*2;", &tokens).unwrap();
    assert!(matches!(
        expression,
        Expression::Binary {
            operator: BinaryOperator::Multiply,
            left,
            ..
        } if matches!(
            *left,
            Expression::Unary {
                operator: UnaryOperator::Negate,
                ..
            }
        )
    ));
}

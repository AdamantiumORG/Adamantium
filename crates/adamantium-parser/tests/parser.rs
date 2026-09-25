#[test]
fn parses_identifier_stream() {
    let tokens = adamantium_lexer::lex("fun main() {}").unwrap();
    let parsed = adamantium_parser::parse("fun main() {}", &tokens);
    assert_eq!(parsed.identifiers[0].name, "main");
    assert_eq!(parsed.span, adamantium_lexer::Span { start: 0, end: 13 });
}

#[test]
fn end_of_input_errors_point_at_the_end_of_the_source() {
    let tokens = adamantium_lexer::lex("value+").unwrap();
    let error = adamantium_parser::parse_expression("value+", &tokens).unwrap_err();
    assert_eq!(error.span, adamantium_lexer::Span::empty_at(6));
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
fn recovery_reports_independent_errors_after_statement_boundaries() {
    let source = "var first = ; var valid = 1; var second += ;";
    let tokens = adamantium_lexer::lex(source).unwrap();
    let output = adamantium_parser::parse_recovering(source, &tokens);

    assert_eq!(output.errors.len(), 2);
    assert_eq!(output.errors[0].message, "expected expression after '='");
    assert_eq!(output.errors[1].message, "expected expression after '+='");
    assert!(
        output
            .file
            .identifiers
            .iter()
            .any(|identifier| identifier.name == "valid")
    );
}

#[test]
fn recovery_synchronizes_at_closing_braces_and_reaches_later_statements() {
    let source = "if condition { var nested = } var later = ;";
    let tokens = adamantium_lexer::lex(source).unwrap();
    let output = adamantium_parser::parse_recovering(source, &tokens);

    assert_eq!(output.errors.len(), 2);
    assert_eq!(
        output.errors[0].span,
        tokens
            .iter()
            .find(|token| token.kind == adamantium_lexer::TokenKind::RightBrace)
            .unwrap()
            .span
    );
    assert!(output.errors[1].span.start > output.errors[0].span.start);
}

#[test]
fn recovery_regression_never_repeats_an_error_at_one_token() {
    for source in [
        "=;=;=;",
        "fun main(){var a=;var b=;}",
        "value + } value * ;",
        "var a = ; } } var b = ;",
    ] {
        let tokens = adamantium_lexer::lex(source).unwrap();
        let output = adamantium_parser::parse_recovering(source, &tokens);
        assert!(
            output.errors.len() <= tokens.len(),
            "recovery stalled for {source:?}"
        );
        assert!(
            output
                .errors
                .windows(2)
                .all(|errors| errors[0].span != errors[1].span)
        );
    }
}

#[test]
fn randomized_utf8_never_panics_parser_recovery() {
    let alphabet = [
        'a', '1', '=', '+', ';', '{', '}', '(', ')', ' ', '\n', 'ż', '🦀',
    ];
    let mut state = 0x5eed_u64;
    for _ in 0..2_000 {
        let length = (next_random(&mut state) % 96) as usize;
        let source = (0..length)
            .map(|_| alphabet[(next_random(&mut state) as usize) % alphabet.len()])
            .collect::<String>();
        let lexed = adamantium_lexer::lex_recovering(&source);
        let result = std::panic::catch_unwind(|| {
            adamantium_parser::parse_recovering(&source, &lexed.tokens)
        });
        assert!(result.is_ok(), "parser recovery panicked for {source:?}");
    }
}

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
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

#[test]
fn token_stream_owns_parser_lookahead_and_expectations() {
    use adamantium_lexer::TokenKind;
    use adamantium_parser::TokenStream;

    let tokens = adamantium_lexer::lex("a+2").unwrap();
    let mut stream = TokenStream::new(tokens.into_iter());
    assert_eq!(stream.peek().unwrap().kind, TokenKind::Identifier);
    assert_eq!(stream.peek_n(1).unwrap().kind, TokenKind::Plus);
    assert_eq!(stream.advance().unwrap().kind, TokenKind::Identifier);
    stream
        .expect(&TokenKind::Plus, "expected '+'")
        .expect("plus should be present");
    assert_eq!(stream.advance().unwrap().kind, TokenKind::IntLiteral);
    assert_eq!(stream.advance().unwrap().kind, TokenKind::Eof);
}

#[test]
fn expression_parser_accepts_any_owned_token_iterator() {
    let source = "a+2";
    let tokens = adamantium_lexer::Lexer::new(source)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert!(adamantium_parser::parse_expression_tokens(source, tokens).is_ok());
}

#[test]
fn pratt_parser_applies_logical_comparison_and_arithmetic_precedence() {
    use adamantium_ast::{BinaryOperator, Expression};

    let source = "a+b*c<d&&e!=f||g==h";
    let tokens = adamantium_lexer::lex(source).unwrap();
    let expression = adamantium_parser::parse_expression(source, &tokens).unwrap();
    let Expression::Binary {
        operator: BinaryOperator::LogicalOr,
        left,
        right,
        ..
    } = expression
    else {
        panic!("logical OR should have the lowest non-assignment precedence");
    };
    assert!(matches!(
        *left,
        Expression::Binary {
            operator: BinaryOperator::LogicalAnd,
            ..
        }
    ));
    assert!(matches!(
        *right,
        Expression::Binary {
            operator: BinaryOperator::Equal,
            ..
        }
    ));
}

#[test]
fn pratt_parser_makes_assignments_right_associative() {
    use adamantium_ast::{BinaryOperator, Expression};

    let source = "a=b+=c*2";
    let tokens = adamantium_lexer::lex(source).unwrap();
    let expression = adamantium_parser::parse_expression(source, &tokens).unwrap();
    let Expression::Binary {
        operator: BinaryOperator::Assign,
        right,
        ..
    } = expression
    else {
        panic!("expected outer assignment");
    };
    assert!(matches!(
        *right,
        Expression::Binary {
            operator: BinaryOperator::AddAssign,
            right,
            ..
        } if matches!(
            *right,
            Expression::Binary {
                operator: BinaryOperator::Multiply,
                ..
            }
        )
    ));
}

#[test]
fn pratt_parser_supports_symbolic_and_word_prefix_and_logical_operators() {
    use adamantium_ast::{BinaryOperator, Expression, UnaryOperator};

    for source in ["!a||b", "not a or b"] {
        let tokens = adamantium_lexer::lex(source).unwrap();
        let expression = adamantium_parser::parse_expression(source, &tokens).unwrap();
        assert!(matches!(
            expression,
            Expression::Binary {
                operator: BinaryOperator::LogicalOr,
                left,
                ..
            } if matches!(
                *left,
                Expression::Unary {
                    operator: UnaryOperator::Not,
                    ..
                }
            )
        ));
    }
}

#[test]
fn pratt_parser_chains_postfix_calls_indexes_and_members() {
    use adamantium_ast::Expression;

    let source = "factory(1,2).items[0].name";
    let tokens = adamantium_lexer::lex(source).unwrap();
    let expression = adamantium_parser::parse_expression(source, &tokens).unwrap();
    let Expression::Member {
        target,
        member,
        span,
    } = expression
    else {
        panic!("expected final member access");
    };
    assert_eq!(member.name, "name");
    assert_eq!((span.start, span.end), (0, source.len() as u32));
    let Expression::Index { target, index, .. } = *target else {
        panic!("expected index before final member");
    };
    assert!(matches!(*index, Expression::Number { .. }));
    let Expression::Member { target, member, .. } = *target else {
        panic!("expected items member before index");
    };
    assert_eq!(member.name, "items");
    assert!(matches!(
        *target,
        Expression::Call { arguments, .. } if arguments.len() == 2
    ));
}

#[test]
fn postfix_parselets_bind_more_tightly_than_infix_operators() {
    use adamantium_ast::{BinaryOperator, Expression};

    let source = "a+b(1)[0].value*c";
    let tokens = adamantium_lexer::lex(source).unwrap();
    let expression = adamantium_parser::parse_expression(source, &tokens).unwrap();
    let Expression::Binary {
        operator: BinaryOperator::Add,
        right,
        ..
    } = expression
    else {
        panic!("expected addition at the root");
    };
    assert!(matches!(
        *right,
        Expression::Binary {
            operator: BinaryOperator::Multiply,
            left,
            ..
        } if matches!(*left, Expression::Member { .. })
    ));
}

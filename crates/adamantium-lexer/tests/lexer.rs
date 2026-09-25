use adamantium_lexer::{Keyword, LexError, Lexer, TokenKind, keyword_kind, lex, lex_recovering};

#[test]
fn classifies_all_keywords_in_one_contract() {
    let cases = [
        ("true", TokenKind::BoolLiteral(true)),
        ("false", TokenKind::BoolLiteral(false)),
        ("and", TokenKind::Keyword(Keyword::And)),
        ("assert", TokenKind::Keyword(Keyword::Assert)),
        ("break", TokenKind::Keyword(Keyword::Break)),
        ("ch", TokenKind::Keyword(Keyword::Changeable)),
        ("changeable", TokenKind::Keyword(Keyword::Changeable)),
        ("class", TokenKind::Keyword(Keyword::Class)),
        ("continue", TokenKind::Keyword(Keyword::Continue)),
        ("define", TokenKind::Keyword(Keyword::Define)),
        ("else", TokenKind::Keyword(Keyword::Else)),
        ("enum", TokenKind::Keyword(Keyword::Enum)),
        ("exit", TokenKind::Keyword(Keyword::Exit)),
        ("for", TokenKind::Keyword(Keyword::For)),
        ("fun", TokenKind::Keyword(Keyword::Fun)),
        ("if", TokenKind::Keyword(Keyword::If)),
        ("implements", TokenKind::Keyword(Keyword::Implements)),
        ("in", TokenKind::Keyword(Keyword::In)),
        ("List", TokenKind::Keyword(Keyword::List)),
        ("loop", TokenKind::Keyword(Keyword::Loop)),
        ("match", TokenKind::Keyword(Keyword::Match)),
        ("None", TokenKind::Keyword(Keyword::None)),
        ("not", TokenKind::Keyword(Keyword::Not)),
        ("offset", TokenKind::Keyword(Keyword::Offset)),
        ("oofset", TokenKind::Keyword(Keyword::Offset)),
        ("or", TokenKind::Keyword(Keyword::Or)),
        ("pack", TokenKind::Keyword(Keyword::Pack)),
        ("panic", TokenKind::Keyword(Keyword::Panic)),
        ("print", TokenKind::Keyword(Keyword::Print)),
        ("priv", TokenKind::Keyword(Keyword::Private)),
        ("pub", TokenKind::Keyword(Keyword::Public)),
        ("return", TokenKind::Keyword(Keyword::Return)),
        ("static", TokenKind::Keyword(Keyword::Static)),
        ("stc", TokenKind::Keyword(Keyword::Static)),
        ("then", TokenKind::Keyword(Keyword::Then)),
        ("trait", TokenKind::Keyword(Keyword::Trait)),
        ("until", TokenKind::Keyword(Keyword::Until)),
        ("use", TokenKind::Keyword(Keyword::Use)),
        ("var", TokenKind::Keyword(Keyword::Variable)),
        ("variable", TokenKind::Keyword(Keyword::Variable)),
        ("warn", TokenKind::Keyword(Keyword::Warn)),
        ("while", TokenKind::Keyword(Keyword::While)),
    ];

    for (text, expected) in cases {
        assert_eq!(keyword_kind(text), Some(expected), "keyword {text:?}");
    }
    assert_eq!(keyword_kind("custom_name"), None);
}

#[test]
fn separates_tokens_without_whitespace() {
    assert_eq!(
        lex("var x=a+b;")
            .unwrap()
            .into_iter()
            .map(|token| token.kind)
            .collect::<Vec<_>>(),
        [
            TokenKind::Keyword(Keyword::Variable),
            TokenKind::Identifier,
            TokenKind::Equals,
            TokenKind::Identifier,
            TokenKind::Plus,
            TokenKind::Identifier,
            TokenKind::Semicolon,
            TokenKind::Eof,
        ]
    );
}

#[test]
fn separates_calls_and_tracks_positions() {
    let tokens = lex("// call\nfoo(a,b)").unwrap();
    assert_eq!(
        tokens.iter().map(|token| &token.kind).collect::<Vec<_>>(),
        [
            &TokenKind::Identifier,
            &TokenKind::LeftParen,
            &TokenKind::Identifier,
            &TokenKind::Comma,
            &TokenKind::Identifier,
            &TokenKind::RightParen,
            &TokenKind::Eof,
        ]
    );
    assert_eq!(tokens[0].span.line, 2);
    assert_eq!(tokens[0].span.column, 1);
}

#[test]
fn exposes_a_streaming_scanner() {
    let mut lexer = Lexer::new("foo+1");
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Identifier);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Plus);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::IntLiteral);
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Eof);
}

#[test]
fn spans_slice_the_exact_utf8_source_text() {
    let source = "var text=\"żółw\";";
    let tokens = lex(source).unwrap();
    assert_eq!(tokens[0].text(source), "var");
    assert_eq!(tokens[1].text(source), "text");
    assert_eq!(tokens[3].text(source), "\"żółw\"");
    assert_eq!(tokens[3].span.start, 9);
    assert_eq!(tokens[3].span.end, 18);
    assert_eq!(tokens.last().unwrap().text(source), "");
}

#[test]
fn operators_use_maximal_munch() {
    let source = "= == != < <= > >= -> => || && += -= *= /= %= .. ::";
    let tokens = lex(source).unwrap();
    assert_eq!(
        tokens.iter().map(|token| &token.kind).collect::<Vec<_>>(),
        [
            &TokenKind::Equals,
            &TokenKind::EqualEqual,
            &TokenKind::NotEqual,
            &TokenKind::Less,
            &TokenKind::LessEqual,
            &TokenKind::Greater,
            &TokenKind::GreaterEqual,
            &TokenKind::Arrow,
            &TokenKind::FatArrow,
            &TokenKind::LogicalOr,
            &TokenKind::LogicalAnd,
            &TokenKind::PlusEqual,
            &TokenKind::MinusEqual,
            &TokenKind::StarEqual,
            &TokenKind::SlashEqual,
            &TokenKind::PercentEqual,
            &TokenKind::Range,
            &TokenKind::DoubleColon,
            &TokenKind::Eof,
        ]
    );
    for token in &tokens[..tokens.len() - 1] {
        assert_eq!(token.text(source).len(), token.span.end - token.span.start);
    }
}

#[test]
fn literal_kinds_are_complete_and_unambiguous() {
    let source = "10 1.5 1e3 true false \"text\"";
    let tokens = lex(source).unwrap();
    assert_eq!(
        tokens.iter().map(|token| &token.kind).collect::<Vec<_>>(),
        [
            &TokenKind::IntLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::BoolLiteral(true),
            &TokenKind::BoolLiteral(false),
            &TokenKind::StringLiteral("text".into()),
            &TokenKind::Eof,
        ]
    );
    assert_eq!(tokens[0].text(source), "10");
    assert_eq!(tokens[2].text(source), "1e3");
}

#[test]
fn recognizes_numeric_suffixes_without_allocating_literal_values() {
    let source = "123 123.0 123.45 123:i32 123:u64 1.5:f64 1e3:f128";
    let tokens = lex(source).unwrap();
    assert_eq!(
        tokens.iter().map(|token| &token.kind).collect::<Vec<_>>(),
        [
            &TokenKind::IntLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::IntLiteral,
            &TokenKind::IntLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::FloatLiteral,
            &TokenKind::Eof,
        ]
    );
    assert_eq!(tokens[3].text(source), "123:i32");
    assert_eq!(tokens[5].text(source), "1.5:f64");
    assert_eq!(tokens[6].text(source), "1e3:f128");
}

#[test]
fn rejects_malformed_numeric_literals_with_their_complete_span() {
    for (source, expected_message) in [
        ("123abc", "must be separated"),
        ("1.2.3", "only one decimal point"),
        ("123:", "expected a type"),
        ("123:wat", "unknown numeric suffix 'wat'"),
        ("1e", "expected exponent digits"),
        ("1e+", "expected exponent digits"),
    ] {
        let error = lex(source).unwrap_err();
        assert!(
            error.message().contains(expected_message),
            "unexpected error for {source:?}: {error}"
        );
        assert_eq!(error.span().start, 0);
        assert_eq!(error.span().end, source.len());
    }
}

#[test]
fn skips_comments_by_default_and_preserves_them_on_request() {
    let source = "var x=10; // explanation\n/* block */ x=x+1;";
    let ordinary = lex(source).unwrap();
    assert!(
        ordinary
            .iter()
            .all(|token| !matches!(token.kind, TokenKind::LineComment | TokenKind::BlockComment))
    );

    let mut lexer = Lexer::with_comments(source);
    let mut preserved = Vec::new();
    loop {
        let token = lexer.next_token().unwrap();
        let finished = token.kind == TokenKind::Eof;
        preserved.push(token);
        if finished {
            break;
        }
    }
    let comments = preserved
        .iter()
        .filter(|token| matches!(token.kind, TokenKind::LineComment | TokenKind::BlockComment))
        .collect::<Vec<_>>();
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].text(source), "// explanation");
    assert_eq!(comments[1].text(source), "/* block */");
    assert_eq!((comments[0].span.line, comments[0].span.column), (1, 11));
    assert_eq!((comments[1].span.line, comments[1].span.column), (2, 1));
}

#[test]
fn comments_never_reach_the_default_parser_token_stream() {
    let source = "var x = 10; // hello\n/* first\n   second */\nprint.newline(x);";
    let tokens = lex(source).unwrap();
    assert_eq!(
        tokens.iter().map(|token| &token.kind).collect::<Vec<_>>(),
        [
            &TokenKind::Keyword(Keyword::Variable),
            &TokenKind::Identifier,
            &TokenKind::Equals,
            &TokenKind::IntLiteral,
            &TokenKind::Semicolon,
            &TokenKind::Keyword(Keyword::Print),
            &TokenKind::Dot,
            &TokenKind::Identifier,
            &TokenKind::LeftParen,
            &TokenKind::Identifier,
            &TokenKind::RightParen,
            &TokenKind::Semicolon,
            &TokenKind::Eof,
        ]
    );
    assert_eq!(tokens[5].text(source), "print");
    assert_eq!((tokens[5].span.line, tokens[5].span.column), (4, 1));
}

#[test]
fn reports_an_unterminated_multiline_comment_with_its_full_span() {
    let source = "/* first\nsecond";
    assert_eq!(
        lex(source).unwrap_err(),
        LexError::UnterminatedComment {
            span: adamantium_lexer::Span {
                start: 0,
                end: source.len(),
                line: 1,
                column: 1,
            }
        }
    );
}

#[test]
fn recovering_lexer_reports_multiple_errors_and_keeps_valid_tokens() {
    let source = "@ var first=1; # \"bad\\q\" var second=2; 123abc";
    let output = lex_recovering(source);

    assert_eq!(output.errors.len(), 4);
    assert!(matches!(
        output.errors[0],
        LexError::UnexpectedCharacter { character: '@', .. }
    ));
    assert!(matches!(
        output.errors[1],
        LexError::UnexpectedCharacter { character: '#', .. }
    ));
    assert!(matches!(
        output.errors[2],
        LexError::InvalidEscape { escape: 'q', .. }
    ));
    assert!(matches!(output.errors[3], LexError::InvalidNumber { .. }));
    assert_eq!(
        output
            .tokens
            .iter()
            .filter(|token| token.kind == TokenKind::Keyword(Keyword::Variable))
            .count(),
        2
    );
    assert_eq!(output.tokens.last().unwrap().kind, TokenKind::Eof);
}

#[test]
fn lexer_diagnostics_never_panic_on_invalid_user_input() {
    for source in ["@", "\"unterminated", "\"bad\\q\"", "/* open", "1e+"] {
        let result = std::panic::catch_unwind(|| lex_recovering(source));
        assert!(result.is_ok(), "lexer panicked for {source:?}");
        assert!(!result.unwrap().errors.is_empty());
    }
}

#[test]
fn comment_markers_inside_strings_remain_string_contents() {
    let source = "\"// text\" \"/* text */\"";
    let tokens = lex(source).unwrap();
    assert_eq!(
        tokens.iter().map(|token| &token.kind).collect::<Vec<_>>(),
        [
            &TokenKind::StringLiteral("// text".into()),
            &TokenKind::StringLiteral("/* text */".into()),
            &TokenKind::Eof,
        ]
    );
}

#[test]
fn decodes_string_escapes_and_unicode() {
    let source = "\"hello\" \"hello\\nworld\" \"quote: \\\"\" \"unicode: żółw\"";
    let tokens = lex(source).unwrap();
    assert_eq!(tokens[0].kind, TokenKind::StringLiteral("hello".into()));
    assert_eq!(
        tokens[1].kind,
        TokenKind::StringLiteral("hello\nworld".into())
    );
    assert_eq!(tokens[2].kind, TokenKind::StringLiteral("quote: \"".into()));
    assert_eq!(
        tokens[3].kind,
        TokenKind::StringLiteral("unicode: żółw".into())
    );
    assert_eq!(tokens[3].text(source), "\"unicode: żółw\"");
}

#[test]
fn returns_structured_string_errors_with_precise_spans() {
    let unterminated = lex("\"unterminated").unwrap_err();
    assert_eq!(
        unterminated,
        LexError::UnterminatedString {
            span: adamantium_lexer::Span {
                start: 0,
                end: 13,
                line: 1,
                column: 1,
            }
        }
    );

    let invalid_escape = lex("\"bad escape \\q\"").unwrap_err();
    assert_eq!(
        invalid_escape,
        LexError::InvalidEscape {
            span: adamantium_lexer::Span {
                start: 12,
                end: 14,
                line: 1,
                column: 13,
            },
            escape: 'q',
        }
    );
    assert!(
        invalid_escape
            .to_string()
            .contains("unsupported string escape")
    );
}

#[test]
fn lexes_literals_comments_and_reports_errors() {
    let tokens = lex("/* x */ 12.5e-2 \"line\\ntext\"").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::FloatLiteral);
    assert_eq!(tokens[0].text("/* x */ 12.5e-2 \"line\\ntext\""), "12.5e-2");
    assert_eq!(
        tokens[1].kind,
        TokenKind::StringLiteral("line\ntext".into())
    );
    assert!(
        lex("/* missing")
            .unwrap_err()
            .message()
            .contains("unterminated")
    );
    assert!(
        lex("\"bad\\q\"")
            .unwrap_err()
            .message()
            .contains("unsupported")
    );
}

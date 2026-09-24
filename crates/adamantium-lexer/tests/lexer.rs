use adamantium_lexer::{Keyword, Lexer, TokenKind, lex};

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
            .message
            .contains("unterminated")
    );
    assert!(
        lex("\"bad\\q\"")
            .unwrap_err()
            .message
            .contains("unsupported")
    );
}

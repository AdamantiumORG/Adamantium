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
            TokenKind::Identifier("x".into()),
            TokenKind::Equals,
            TokenKind::Identifier("a".into()),
            TokenKind::Plus,
            TokenKind::Identifier("b".into()),
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
            &TokenKind::Identifier("foo".into()),
            &TokenKind::LeftParen,
            &TokenKind::Identifier("a".into()),
            &TokenKind::Comma,
            &TokenKind::Identifier("b".into()),
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
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Identifier("foo".into())
    );
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Plus);
    assert_eq!(
        lexer.next_token().unwrap().kind,
        TokenKind::Number("1".into())
    );
    assert_eq!(lexer.next_token().unwrap().kind, TokenKind::Eof);
}

#[test]
fn lexes_literals_comments_and_reports_errors() {
    let tokens = lex("/* x */ 12.5e-2 \"line\\ntext\"").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Number("12.5e-2".into()));
    assert_eq!(tokens[1].kind, TokenKind::String("line\ntext".into()));
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

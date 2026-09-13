#[test]
fn lexes_words_with_lines() {
    let tokens = adamantium_lexer::lex_words("fun main\nprint");
    assert_eq!(tokens[2].span.line, 2);
}

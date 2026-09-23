#[test]
fn parses_identifier_stream() {
    let tokens = adamantium_lexer::lex("fun main() {}").unwrap();
    let parsed = adamantium_parser::parse(&tokens);
    assert_eq!(parsed.identifiers[0].name, "main");
}

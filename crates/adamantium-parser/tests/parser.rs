#[test]
fn parses_identifier_stream() {
    assert_eq!(adamantium_parser::parse("fun main").identifiers.len(), 2);
}

#[test]
fn parses_identifier_stream() {
    let parsed = adamantium_parser::parse("fun main() {}").unwrap();
    assert_eq!(parsed.identifiers[0].name, "main");
}

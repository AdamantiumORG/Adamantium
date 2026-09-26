#[test]
fn public_formatter_is_deterministic_and_idempotent() {
    let source = "fun main(){var value=1+2;print.newline(value);}";
    let formatted = adamantium_fmt::format_source(source).unwrap();
    assert_eq!(
        adamantium_fmt::format_source(&formatted).unwrap(),
        formatted
    );
    assert!(formatted.contains("var value = 1 + 2;"));
}

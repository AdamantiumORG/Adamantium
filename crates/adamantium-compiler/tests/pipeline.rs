#[test]
fn connects_compiler_layers() {
    assert!(
        adamantium_compiler::architecture_smoke_test("fun main")
            .unwrap()
            .contains("ret")
    );
}

#[test]
fn reports_all_lexer_errors_in_one_compiler_pass() {
    let diagnostics = adamantium_compiler::architecture_smoke_test("@ fun main #")
        .expect_err("invalid characters should produce diagnostics");
    assert_eq!(diagnostics.len(), 2);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == "E100")
    );
    assert_eq!(diagnostics[0].span.start, 0);
    assert_eq!(diagnostics[1].span.start, 11);
}

#[test]
fn keeps_lexer_and_parser_diagnostics_separate() {
    let lexical = adamantium_compiler::architecture_smoke_test("var x = @;").unwrap_err();
    assert_eq!(lexical.len(), 1);
    assert_eq!(lexical[0].code, "E100");
    assert!(lexical[0].message.contains("unexpected character '@'"));

    let syntax = adamantium_compiler::architecture_smoke_test("var x = ;").unwrap_err();
    assert_eq!(syntax.len(), 1);
    assert_eq!(syntax[0].code, "E110");
    assert_eq!(syntax[0].message, "expected expression after '='");
}

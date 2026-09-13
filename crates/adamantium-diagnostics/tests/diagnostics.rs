#[test]
fn creates_coded_error() {
    let diagnostic = adamantium_diagnostics::Diagnostic::error("E001", "bad", Default::default());
    assert_eq!(diagnostic.code, "E001");
}

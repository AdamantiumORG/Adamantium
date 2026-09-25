#[test]
fn creates_coded_error() {
    let diagnostic = adamantium_diagnostics::Diagnostic::error("E001", "bad", Default::default());
    assert_eq!(diagnostic.code, "E001");
}

#[test]
fn carries_the_complete_cross_phase_diagnostic_contract() {
    use adamantium_diagnostics::{Diagnostic, Severity, Stage};

    let diagnostic = Diagnostic::at_stage(
        Stage::TypeChecking,
        Severity::Error,
        "E0308",
        "mismatched types",
        adamantium_ast::Span { start: 17, end: 24 },
    )
    .with_primary_message("expected i32, found string")
    .with_secondary(
        adamantium_ast::Span { start: 4, end: 5 },
        "variable declared as i32 here",
    )
    .with_note("assignments preserve the declared variable type")
    .with_help("convert the value to i32 before assigning it");

    assert_eq!(diagnostic.stage, Stage::TypeChecking);
    assert_eq!(
        diagnostic.span(),
        adamantium_ast::Span { start: 17, end: 24 }
    );
    assert_eq!(diagnostic.secondary.len(), 1);
    assert_eq!(diagnostic.notes.len(), 1);
    assert!(diagnostic.help.is_some());
}

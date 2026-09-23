pub fn analyze(tokens: &[adamantium_lexer::Token]) -> Vec<adamantium_diagnostics::Diagnostic> {
    let parsed = adamantium_parser::parse(tokens);
    let _default_type = adamantium_types::infer_literal("0", Default::default());
    let mut seen = std::collections::HashSet::new();
    parsed
        .identifiers
        .into_iter()
        .filter_map(|identifier| {
            if seen.insert(identifier.name.clone()) {
                None
            } else {
                Some(adamantium_diagnostics::Diagnostic::error(
                    "E100",
                    format!("duplicate identifier '{}'", identifier.name),
                    identifier.span,
                ))
            }
        })
        .collect()
}

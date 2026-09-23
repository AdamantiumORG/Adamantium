pub fn analyze(source: &str) -> Vec<adamantium_diagnostics::Diagnostic> {
    let parsed = match adamantium_parser::parse(source) {
        Ok(parsed) => parsed,
        Err(error) => {
            return vec![adamantium_diagnostics::Diagnostic::error(
                "E100",
                error.message,
                error.span,
            )];
        }
    };
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

pub fn analyze(
    source: &str,
    tokens: &[adamantium_lexer::Token],
) -> Vec<adamantium_diagnostics::Diagnostic> {
    let parsed = adamantium_parser::parse(source, tokens);
    let _default_type = adamantium_types::infer_literal("0");
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

pub struct Resolution {
    pub span: adamantium_lexer::Span,
    pub definitions: Vec<ResolvedDefinition>,
    pub symbols: adamantium_hir::SymbolInterner,
}

pub struct ResolvedDefinition {
    pub id: adamantium_hir::DefId,
    pub symbol: adamantium_hir::SymbolId,
    pub span: adamantium_lexer::Span,
}

pub fn resolve(
    parsed: &adamantium_parser::ParsedFile,
) -> Result<Resolution, Vec<adamantium_diagnostics::Diagnostic>> {
    let mut symbols = adamantium_hir::SymbolInterner::default();
    let mut definitions = Vec::new();
    let mut defined = std::collections::HashMap::new();
    let mut diagnostics = Vec::new();
    for identifier in &parsed.identifiers {
        let symbol = symbols.intern(&identifier.name);
        if defined.contains_key(&symbol) {
            diagnostics.push(adamantium_diagnostics::Diagnostic::error(
                "E200",
                format!("duplicate identifier '{}'", identifier.name),
                identifier.span,
            ));
            continue;
        }
        let id =
            adamantium_hir::DefId(u32::try_from(definitions.len()).expect("too many definitions"));
        defined.insert(symbol, id);
        definitions.push(ResolvedDefinition {
            id,
            symbol,
            span: identifier.span,
        });
    }
    if diagnostics.is_empty() {
        Ok(Resolution {
            span: parsed.span,
            definitions,
            symbols,
        })
    } else {
        Err(diagnostics)
    }
}

pub fn type_check(
    resolution: &Resolution,
    types: &mut adamantium_types::TypeInterner,
) -> adamantium_hir::Program {
    let default_type = types.intern(adamantium_types::TypeKind::Primitive(
        adamantium_types::Type::I32,
    ));
    adamantium_hir::Program {
        span: resolution.span,
        definitions: resolution
            .definitions
            .iter()
            .map(|definition| adamantium_hir::Definition {
                id: definition.id,
                symbol: definition.symbol,
                span: definition.span,
                ty: default_type,
            })
            .collect(),
    }
}

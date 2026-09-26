pub fn pipeline_layers() -> &'static [&'static str] {
    &[
        "lexer",
        "parser",
        "ast",
        "name-resolution",
        "type-checking",
        "hir",
        "mir",
        "ir",
        "codegen",
        "nasm",
        "linker",
    ]
}

pub fn architecture_smoke_test(
    source: &str,
) -> Result<String, Vec<adamantium_diagnostics::Diagnostic>> {
    let lexed = adamantium_lexer::lex_recovering(source);
    let diagnostics = frontend_diagnostics_from_lexed(source, &lexed);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let tokens = lexed.tokens;
    let parsed = adamantium_parser::parse_checked(source, &tokens).map_err(|errors| {
        errors
            .into_iter()
            .map(|error| {
                adamantium_diagnostics::Diagnostic::at_stage(
                    adamantium_diagnostics::Stage::Parsing,
                    adamantium_diagnostics::Severity::Error,
                    "E110",
                    error.message,
                    error.span,
                )
            })
            .collect::<Vec<_>>()
    })?;
    let mut types = adamantium_types::TypeInterner::default();
    let resolution = adamantium_semantics::resolve(&parsed)?;
    let hir = adamantium_semantics::type_check(&resolution, &mut types);
    let none = types.intern(adamantium_types::TypeKind::Primitive(
        adamantium_types::Type::None,
    ));
    let mir = adamantium_lowering::hir_to_mir(&hir, none);
    let ir = adamantium_lowering::mir_to_ir(&mir);
    let assembly = adamantium_codegen::emit_program(&ir);
    let _object = adamantium_nasm::object_path(std::path::Path::new("main.asm"), cfg!(windows));
    let _executable = adamantium_linker::executable_name("main", cfg!(windows));
    let _project = adamantium_project::ProjectLayout::new(".");
    let _asset = adamantium_packages::release_asset("0.1.0");
    let _wasm = adamantium_wasm::is_module(adamantium_wasm::MAGIC);
    let _type = adamantium_types::PrimitiveType::Int;
    Ok(assembly)
}

/// Runs the recovery-capable lexical and grammar checks used before the mature
/// semantic frontend. Every independent error is returned in source order.
pub fn frontend_diagnostics(source: &str) -> Vec<adamantium_diagnostics::Diagnostic> {
    let lexed = adamantium_lexer::lex_recovering(source);
    frontend_diagnostics_from_lexed(source, &lexed)
}

fn frontend_diagnostics_from_lexed(
    source: &str,
    lexed: &adamantium_lexer::LexOutput,
) -> Vec<adamantium_diagnostics::Diagnostic> {
    if !lexed.errors.is_empty() {
        return lexed
            .errors
            .iter()
            .map(|error| {
                adamantium_diagnostics::Diagnostic::at_stage(
                    adamantium_diagnostics::Stage::Lexing,
                    adamantium_diagnostics::Severity::Error,
                    "E100",
                    error.message(),
                    error.span(),
                )
            })
            .collect();
    }
    adamantium_parser::parse_recovering(source, &lexed.tokens)
        .errors
        .into_iter()
        .map(|error| {
            adamantium_diagnostics::Diagnostic::at_stage(
                adamantium_diagnostics::Stage::Parsing,
                adamantium_diagnostics::Severity::Error,
                "E110",
                error.message,
                error.span,
            )
        })
        .collect()
}

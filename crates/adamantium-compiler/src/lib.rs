pub fn pipeline_layers() -> &'static [&'static str] {
    &["parser", "semantics", "ir", "codegen", "nasm", "linker"]
}

pub fn architecture_smoke_test(
    source: &str,
) -> Result<String, Vec<adamantium_diagnostics::Diagnostic>> {
    let tokens = adamantium_lexer::lex(source).map_err(|error| {
        vec![adamantium_diagnostics::Diagnostic::error(
            "E100",
            error.message,
            error.span,
        )]
    })?;
    let _parsed = adamantium_parser::parse(source, &tokens);
    let diagnostics = adamantium_semantics::analyze(source, &tokens);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let assembly = adamantium_codegen::emit(&[
        adamantium_ir::integer(0),
        adamantium_ir::Instruction::Return,
    ]);
    let _object = adamantium_nasm::object_path(std::path::Path::new("main.asm"), cfg!(windows));
    let _executable = adamantium_linker::executable_name("main", cfg!(windows));
    let _project = adamantium_project::ProjectLayout::new(".");
    let _asset = adamantium_packages::release_asset("0.1.0");
    let _wasm = adamantium_wasm::is_module(adamantium_wasm::MAGIC);
    let _type = adamantium_types::PrimitiveType::Int;
    Ok(assembly)
}

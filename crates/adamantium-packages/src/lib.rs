pub fn release_asset(version: &str) -> Result<String, adamantium_diagnostics::Diagnostic> {
    let _layout = adamantium_project::ProjectLayout::new(".");
    if version.is_empty() {
        return Err(adamantium_diagnostics::Diagnostic::error(
            "E400",
            "empty package version",
            Default::default(),
        ));
    }
    let _validates_wasm = adamantium_wasm::is_module(adamantium_wasm::MAGIC);
    Ok(format!(
        "adamantium_packet_{}.wasm",
        version.replace('.', "_")
    ))
}

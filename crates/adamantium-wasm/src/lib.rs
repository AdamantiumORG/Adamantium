pub const MAGIC: &[u8; 4] = b"\0asm";

pub const ABI: &str = "wasi-command-v1";
pub const WASI_MODULE: &str = "wasi_snapshot_preview1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModuleInfo {
    pub exported_functions: Vec<String>,
    pub imports_wasi: bool,
}

pub fn is_module(bytes: &[u8]) -> bool {
    let _abi_type = adamantium_types::PrimitiveType::Int;
    wasmparser::Validator::new().validate_all(bytes).is_ok()
}

pub fn validate_package(bytes: &[u8]) -> Result<ModuleInfo, String> {
    use wasmparser::{Encoding, ExternalKind, Parser, Payload, TypeRef, Validator};

    Validator::new()
        .validate_all(bytes)
        .map_err(|error| format!("invalid WebAssembly module: {error}"))?;
    let mut functions = Vec::new();
    let mut imports_wasi = false;
    for payload in Parser::new(0).parse_all(bytes) {
        match payload.map_err(|error| format!("invalid WebAssembly module: {error}"))? {
            Payload::Version { encoding, .. } if encoding != Encoding::Module => {
                return Err("WebAssembly components cannot be used as Adamantium packages".into());
            }
            Payload::ImportSection(section) => {
                for import in section {
                    let import = import.map_err(|error| format!("invalid WASM import: {error}"))?;
                    if import.module != WASI_MODULE {
                        return Err(format!(
                            "unsupported WASM import module '{}'; only {WASI_MODULE} is allowed",
                            import.module
                        ));
                    }
                    imports_wasi = true;
                    if matches!(import.ty, TypeRef::Tag(_)) {
                        return Err(
                            "WASM exception tags are not supported by the package ABI".into()
                        );
                    }
                }
            }
            Payload::ExportSection(section) => {
                for export in section {
                    let export = export.map_err(|error| format!("invalid WASM export: {error}"))?;
                    if export.kind == ExternalKind::Func {
                        functions.push(export.name.to_string());
                    }
                }
            }
            _ => {}
        }
    }
    functions.sort();
    functions.dedup();
    if !functions.iter().any(|name| name == "_start") {
        return Err("wasi-command-v1 package must export a '_start' function".into());
    }
    Ok(ModuleInfo {
        exported_functions: functions,
        imports_wasi,
    })
}

pub const MAGIC: &[u8; 4] = b"\0asm";

pub const ABI: &str = "wasi-command-v1";
pub const WASI_MODULE: &str = "wasi_snapshot_preview1";
pub const MAX_MODULE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_MEMORY_PAGES: u64 = 4096;

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

    if bytes.len() > MAX_MODULE_BYTES {
        return Err(format!(
            "package module is {} bytes; the limit is {MAX_MODULE_BYTES} bytes",
            bytes.len()
        ));
    }
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
                    if let TypeRef::Memory(memory) = import.ty {
                        validate_memory(memory)?;
                    }
                }
            }
            Payload::MemorySection(section) => {
                for memory in section {
                    validate_memory(
                        memory.map_err(|error| format!("invalid WASM memory: {error}"))?,
                    )?;
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

fn validate_memory(memory: wasmparser::MemoryType) -> Result<(), String> {
    if memory.memory64 {
        return Err("memory64 is not supported by the package ABI".into());
    }
    if memory.initial > MAX_MEMORY_PAGES {
        return Err(format!(
            "package memory starts at {} pages; the limit is {MAX_MEMORY_PAGES}",
            memory.initial
        ));
    }
    match memory.maximum {
        Some(maximum) if maximum <= MAX_MEMORY_PAGES => Ok(()),
        Some(maximum) => Err(format!(
            "package memory can grow to {maximum} pages; the limit is {MAX_MEMORY_PAGES}"
        )),
        None => Ok(()),
    }
}

use adamantium_wasm::{WASI_MODULE, validate_package};

#[test]
fn validates_wasi_command_and_discovers_exports() {
    let bytes = wat::parse_str(
        r#"(module
            (import "wasi_snapshot_preview1" "proc_exit" (func (param i32)))
            (func (export "helper"))
            (func (export "_start")))"#,
    )
    .unwrap();
    let info = validate_package(&bytes).unwrap();
    assert!(info.imports_wasi);
    assert_eq!(info.exported_functions, ["_start", "helper"]);
    assert_eq!(WASI_MODULE, "wasi_snapshot_preview1");
}

#[test]
fn rejects_malformed_modules_and_missing_start() {
    assert!(validate_package(b"\0asm\x01\0\0\0broken").is_err());
    let no_start = wat::parse_str("(module (func (export \"other\")))").unwrap();
    assert!(validate_package(&no_start).unwrap_err().contains("_start"));
}

#[test]
fn rejects_non_wasi_import_namespaces() {
    let bytes = wat::parse_str(
        r#"(module
            (import "env" "host_function" (func))
            (func (export "_start")))"#,
    )
    .unwrap();
    assert!(
        validate_package(&bytes)
            .unwrap_err()
            .contains("only wasi_snapshot_preview1")
    );
}

#[test]
fn enforces_package_memory_limits() {
    let unbounded = wat::parse_str("(module (memory 1) (func (export \"_start\")))").unwrap();
    validate_package(&unbounded).unwrap();

    let oversized = wat::parse_str("(module (memory 1 4097) (func (export \"_start\")))").unwrap();
    assert!(
        validate_package(&oversized)
            .unwrap_err()
            .contains("4097 pages")
    );

    let bounded = wat::parse_str("(module (memory 1 4096) (func (export \"_start\")))").unwrap();
    validate_package(&bounded).unwrap();
}

#[test]
fn rejects_oversized_package_files_before_parsing() {
    let bytes = vec![0; adamantium_wasm::MAX_MODULE_BYTES + 1];
    assert!(
        validate_package(&bytes)
            .unwrap_err()
            .contains("the limit is")
    );
}

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        // Exercises lexer -> recovery parser -> name resolution -> type checking
        // -> HIR -> MIR -> IR -> code generation. Diagnostics are valid output;
        // a panic, abort, or memory error is not.
        let _ = adamantium_compiler::architecture_smoke_test(source);

        // Keep the mature production frontend in the same corpus until its
        // remaining implementation has moved behind the compiler crate APIs.
        let _ = adamantium_cli::check_frontend_input(source, false);
        let _ = adamantium_cli::check_frontend_input(source, true);
    }
});

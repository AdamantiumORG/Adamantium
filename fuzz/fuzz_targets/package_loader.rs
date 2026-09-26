#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = adamantium_wasm::is_module(data);
    let _ = adamantium_wasm::validate_package(data);
});

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(source) = std::str::from_utf8(data) {
        let _ = adamantium_cli::check_frontend_input(source, false);
        let _ = adamantium_cli::check_frontend_input(source, true);
    }
});

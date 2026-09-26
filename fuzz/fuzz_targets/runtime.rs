#![no_main]

use adamantium_runtime::types::{Type, Value};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut bytes = [0; 16];
    let length = data.len().min(bytes.len());
    bytes[..length].copy_from_slice(&data[..length]);
    let value = Value::from_bits(u128::from_le_bytes(bytes));
    for ty in [
        Type::I8,
        Type::I16,
        Type::I32,
        Type::I64,
        Type::U8,
        Type::U16,
        Type::U32,
        Type::U64,
    ] {
        let _ = value.integer(ty);
        let _ = ty.bounds();
        let _ = Type::from_id(ty.id());
    }
});

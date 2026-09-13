pub const MAGIC: &[u8; 4] = b"\0asm";

pub fn is_module(bytes: &[u8]) -> bool {
    let _abi_type = adamantium_types::PrimitiveType::Int;
    bytes.starts_with(MAGIC)
}

#![allow(dead_code)]

#[allow(dead_code)]
mod types {
    pub use adamantium_types::*;
}
#[cfg(test)]
mod codegen;
mod syntax;
#[cfg(test)]
mod typed;

/// Exercises the complete source frontend without performing file-system or
/// native-toolchain operations. Fuzz targets use this stable boundary.
pub fn check_frontend_input(source: &str, professional: bool) -> Result<(), String> {
    if professional {
        syntax::validate_professional(source)?;
    }
    syntax::parse_modules_with_mode(&[(String::new(), source.to_owned())], professional).map(drop)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arbitrary_user_input_never_panics_the_frontend() {
        let alphabet = b"abcdefghijklmnopqrstuvwxyz0123456789_{}()[];:=,+-*/%!<>|&$\"\n\0\xff";
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        for case in 0..10_000 {
            state ^= state << 7;
            state ^= state >> 9;
            state ^= state << 8;
            let length = (state as usize % 128) + 1;
            let mut bytes = Vec::with_capacity(length);
            for _ in 0..length {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                bytes.push(alphabet[state as usize % alphabet.len()]);
            }
            let source = String::from_utf8_lossy(&bytes);
            for professional in [false, true] {
                let result = std::panic::catch_unwind(|| {
                    let _ = check_frontend_input(&source, professional);
                });
                assert!(result.is_ok(), "frontend panicked for corpus case {case}");
            }
        }
    }
}

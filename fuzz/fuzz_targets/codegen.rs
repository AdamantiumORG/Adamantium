#![no_main]

use adamantium_codegen::{Arm64Target, emit, emit_llvm_arm64};
use adamantium_ir::{Instruction, Program, Span, SpannedInstruction};
use adamantium_types::PrimitiveType;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let instructions = data
        .chunks(9)
        .take(4096)
        .map(|chunk| {
            if chunk.first().copied().unwrap_or_default() & 1 == 0 {
                let mut bytes = [0; 8];
                let value = chunk.get(1..).unwrap_or_default();
                bytes[..value.len()].copy_from_slice(value);
                Instruction::Constant {
                    value: i64::from_le_bytes(bytes),
                    ty: PrimitiveType::Int,
                }
            } else {
                Instruction::Return
            }
        })
        .collect::<Vec<_>>();
    let program = Program {
        span: Span {
            start: 0,
            end: data.len() as u32,
        },
        instructions: instructions
            .iter()
            .cloned()
            .map(|instruction| SpannedInstruction {
                instruction,
                span: Span {
                    start: 0,
                    end: data.len() as u32,
                },
            })
            .collect(),
    };
    let _ = emit(&instructions);
    let _ = emit_llvm_arm64(&program, Arm64Target::Windows);
    let _ = emit_llvm_arm64(&program, Arm64Target::Linux);
});

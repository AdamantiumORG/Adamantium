use adamantium_codegen::{Arm64Target, emit_llvm_arm64};
use adamantium_ir::{Program, Span, SpannedInstruction, integer};
use std::path::PathBuf;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os().skip(1);
    let target = match arguments
        .next()
        .ok_or("expected target")?
        .to_string_lossy()
        .as_ref()
    {
        "aarch64-pc-windows-msvc" => Arm64Target::Windows,
        "aarch64-unknown-linux-gnu" => Arm64Target::Linux,
        value => return Err(format!("unsupported ARM64 target '{value}'")),
    };
    let output = PathBuf::from(arguments.next().ok_or("expected output path")?);
    if arguments.next().is_some() {
        return Err("too many arguments".into());
    }
    let span = Span { start: 0, end: 0 };
    let program = Program {
        span,
        instructions: vec![
            SpannedInstruction {
                instruction: integer(0),
                span,
            },
            SpannedInstruction {
                instruction: adamantium_ir::Instruction::Return,
                span,
            },
        ],
    };
    std::fs::write(output, emit_llvm_arm64(&program, target).assembly)
        .map_err(|error| error.to_string())
}

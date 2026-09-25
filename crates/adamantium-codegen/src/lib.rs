pub fn emit(instructions: &[adamantium_ir::Instruction]) -> String {
    let _integer_type = adamantium_types::PrimitiveType::Int;
    instructions
        .iter()
        .map(|instruction| match instruction {
            adamantium_ir::Instruction::Constant { value, .. } => format!("mov rax, {value}\n"),
            adamantium_ir::Instruction::Return => "ret\n".to_owned(),
        })
        .collect()
}

pub fn emit_program(program: &adamantium_ir::Program) -> String {
    emit_program_with_spans(program).assembly
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmittedProgram {
    pub assembly: String,
    pub instruction_spans: Vec<adamantium_ir::Span>,
}

pub fn emit_program_with_spans(program: &adamantium_ir::Program) -> EmittedProgram {
    let instructions = program
        .instructions
        .iter()
        .map(|instruction| instruction.instruction.clone())
        .collect::<Vec<_>>();
    EmittedProgram {
        assembly: emit(&instructions),
        instruction_spans: program
            .instructions
            .iter()
            .map(|instruction| instruction.span)
            .collect(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Arm64Target {
    Windows,
    Linux,
}

impl Arm64Target {
    pub const fn triple(self) -> &'static str {
        match self {
            Self::Windows => "aarch64-pc-windows-msvc",
            Self::Linux => "aarch64-unknown-linux-gnu",
        }
    }

    pub const fn data_layout(self) -> &'static str {
        match self {
            Self::Windows => "e-m:w-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-n32:64-S128",
            Self::Linux => "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128",
        }
    }

    pub const fn object_extension(self) -> &'static str {
        match self {
            Self::Windows => "obj",
            Self::Linux => "o",
        }
    }
}

/// Emit target-specific LLVM IR from the target-independent core IR.
///
/// The LLVM module uses a C-compatible `main` entry point and never contains
/// x86 instructions or host-dependent pointer constants.
pub fn emit_llvm_arm64(program: &adamantium_ir::Program, target: Arm64Target) -> EmittedProgram {
    let mut body = String::new();
    let mut value = 0i64;
    for instruction in &program.instructions {
        match instruction.instruction {
            adamantium_ir::Instruction::Constant {
                value: constant, ..
            } => value = constant,
            adamantium_ir::Instruction::Return => {
                body.push_str(&format!("  ret i32 {}\n", value as i32));
            }
        }
    }
    if !body.contains("ret ") {
        body.push_str("  ret i32 0\n");
    }
    EmittedProgram {
        assembly: format!(
            "; Adamantium LLVM ARM64 module\n\
             target datalayout = \"{}\"\n\
             target triple = \"{}\"\n\n\
             define i32 @main(i32 %argc, ptr %argv) {{\n\
             entry:\n\
             {}\
             }}\n",
            target.data_layout(),
            target.triple(),
            body
        ),
        instruction_spans: program
            .instructions
            .iter()
            .map(|instruction| instruction.span)
            .collect(),
    }
}

pub fn llvm_object_command(
    clang: impl Into<std::path::PathBuf>,
    input: &std::path::Path,
    output: &std::path::Path,
    target: Arm64Target,
) -> std::process::Command {
    let mut command = std::process::Command::new(clang.into());
    command
        .arg("-target")
        .arg(target.triple())
        .arg("-c")
        .arg(input)
        .arg("-o")
        .arg(output);
    command
}

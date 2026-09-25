use adamantium_codegen::{Arm64Target, emit_llvm_arm64, llvm_object_command};
use adamantium_ir::{Program, Span, SpannedInstruction, integer};
use std::path::Path;

fn program() -> Program {
    Program {
        span: Span { start: 0, end: 8 },
        instructions: vec![
            SpannedInstruction {
                instruction: integer(7),
                span: Span { start: 0, end: 1 },
            },
            SpannedInstruction {
                instruction: adamantium_ir::Instruction::Return,
                span: Span { start: 2, end: 8 },
            },
        ],
    }
}

#[test]
fn emits_windows_and_linux_arm64_llvm_modules() {
    for target in [Arm64Target::Windows, Arm64Target::Linux] {
        let emitted = emit_llvm_arm64(&program(), target);
        assert!(emitted.assembly.contains(target.triple()));
        assert!(emitted.assembly.contains(target.data_layout()));
        assert!(emitted.assembly.contains("define i32 @main"));
        assert!(emitted.assembly.contains("ret i32 7"));
        assert_eq!(emitted.instruction_spans.len(), 2);
        assert!(!emitted.assembly.contains("rax"));
    }
}

#[test]
fn configures_clang_for_each_arm64_object_format() {
    for target in [Arm64Target::Windows, Arm64Target::Linux] {
        let command =
            llvm_object_command("clang", Path::new("main.ll"), Path::new("main.o"), target);
        let arguments = command
            .get_args()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(arguments[0..2], ["-target", target.triple()]);
        assert_eq!(
            target.object_extension(),
            if target == Arm64Target::Windows {
                "obj"
            } else {
                "o"
            }
        );
    }
}

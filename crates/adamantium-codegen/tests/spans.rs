use adamantium_ir::{Instruction, Program, Span, SpannedInstruction};

#[test]
fn keeps_instruction_spans_next_to_emitted_assembly() {
    let source_span = Span { start: 8, end: 14 };
    let program = Program {
        span: Span { start: 0, end: 14 },
        instructions: vec![SpannedInstruction {
            instruction: Instruction::Return,
            span: source_span,
        }],
    };

    let emitted = adamantium_codegen::emit_program_with_spans(&program);
    assert_eq!(emitted.assembly, "ret\n");
    assert_eq!(emitted.instruction_spans, vec![source_span]);
}

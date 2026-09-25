use adamantium_types::TypeId;

pub fn hir_to_mir(hir: &adamantium_hir::Program, none: TypeId) -> adamantium_mir::Program {
    adamantium_mir::lower(hir, none)
}

pub fn mir_to_ir(mir: &adamantium_mir::Program) -> adamantium_ir::Program {
    adamantium_ir::Program {
        instructions: mir
            .instructions
            .iter()
            .filter_map(|instruction| match instruction.kind {
                adamantium_mir::InstructionKind::Define(_) => None,
                adamantium_mir::InstructionKind::Return => {
                    Some(adamantium_ir::SpannedInstruction {
                        instruction: adamantium_ir::Instruction::Return,
                        span: instruction.span,
                    })
                }
            })
            .collect(),
    }
}

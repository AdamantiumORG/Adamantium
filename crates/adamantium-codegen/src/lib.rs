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
    let instructions = program
        .instructions
        .iter()
        .map(|instruction| instruction.instruction.clone())
        .collect::<Vec<_>>();
    emit(&instructions)
}

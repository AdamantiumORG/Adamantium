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

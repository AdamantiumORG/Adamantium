use adamantium_types::PrimitiveType;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Instruction {
    Constant { value: i64, ty: PrimitiveType },
    Return,
}

pub fn integer(value: i64) -> Instruction {
    Instruction::Constant {
        value,
        ty: PrimitiveType::Int,
    }
}

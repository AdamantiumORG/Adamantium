use adamantium_ast::Span;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrimitiveType {
    Int,
    Float,
    Unsigned,
    String,
    None,
}

pub fn infer_literal(text: &str, _span: Span) -> PrimitiveType {
    if text == "None" {
        PrimitiveType::None
    } else if text.starts_with('"') {
        PrimitiveType::String
    } else if text.contains('.') {
        PrimitiveType::Float
    } else {
        PrimitiveType::Int
    }
}

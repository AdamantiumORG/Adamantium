pub use adamantium_lexer::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

impl Identifier {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Identifier(Identifier),
    Number {
        literal: String,
        span: Span,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Self::Identifier(identifier) => identifier.span,
            Self::Number { span, .. } | Self::Binary { span, .. } => *span,
        }
    }
}

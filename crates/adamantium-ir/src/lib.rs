use adamantium_types::PrimitiveType;
use std::collections::HashMap;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpannedInstruction {
    pub instruction: Instruction,
    pub span: adamantium_lexer::Span,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Program {
    pub instructions: Vec<SpannedInstruction>,
}

pub mod typed {
    use super::HashMap;

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ArithmeticOperator {
        Add,
        Subtract,
        Multiply,
        Divide,
        Remainder,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum ComparisonOperator {
        Equal,
        NotEqual,
        Less,
        LessEqual,
        Greater,
        GreaterEqual,
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum LogicalOperator {
        And,
        Or,
    }

    #[derive(Clone)]
    pub struct Expression<T, V> {
        pub ty: T,
        pub kind: ExpressionKind<T, V>,
    }

    #[derive(Clone)]
    pub enum ExpressionKind<T, V> {
        Constant(V),
        String(Vec<u8>),
        Variable(usize),
        Negate(Box<Expression<T, V>>),
        Not(Box<Expression<T, V>>),
        Binary(
            ArithmeticOperator,
            Box<Expression<T, V>>,
            Box<Expression<T, V>>,
        ),
        Compare(
            ComparisonOperator,
            Box<Expression<T, V>>,
            Box<Expression<T, V>>,
        ),
        Logical(
            LogicalOperator,
            Box<Expression<T, V>>,
            Box<Expression<T, V>>,
        ),
        Convert(Box<Expression<T, V>>),
        Unwrap(Box<Expression<T, V>>, usize),
        Call(String, Vec<Expression<T, V>>),
        Construct(u32, Vec<Expression<T, V>>, String),
        List(Vec<Expression<T, V>>),
        StringLength(Box<Expression<T, V>>, usize),
        Index(Box<Expression<T, V>>, Box<Expression<T, V>>, usize),
        Field(Box<Expression<T, V>>, usize),
        MethodCall(String, Box<Expression<T, V>>, Vec<Expression<T, V>>),
        Try(Vec<Statement<T, V>>),
        Address(usize),
        Dereference(Box<Expression<T, V>>),
    }

    pub type MatchBranch<T, V> = (Expression<T, V>, Vec<Statement<T, V>>);

    #[derive(Clone)]
    pub enum Statement<T, V> {
        Noop,
        Assign(usize, Expression<T, V>),
        Disconnect(usize, usize),
        Remove(Expression<T, V>, Option<String>),
        Clamp(usize, Expression<T, V>, Expression<T, V>),
        Print(Expression<T, V>, bool),
        Call(Expression<T, V>),
        SetField(Expression<T, V>, usize, Expression<T, V>, Option<String>),
        SetIndex(Expression<T, V>, Expression<T, V>, Expression<T, V>, usize),
        Message(Expression<T, V>, bool, usize),
        If(Expression<T, V>, Vec<Statement<T, V>>, Vec<Statement<T, V>>),
        While(Expression<T, V>, Vec<Statement<T, V>>),
        Until(Expression<T, V>, Vec<Statement<T, V>>),
        Loop(Vec<Statement<T, V>>),
        For(
            usize,
            Expression<T, V>,
            Expression<T, V>,
            Vec<Statement<T, V>>,
        ),
        ForEach(usize, Expression<T, V>, Vec<Statement<T, V>>),
        Match(
            Expression<T, V>,
            Vec<MatchBranch<T, V>>,
            Option<Vec<Statement<T, V>>>,
        ),
        Break,
        Continue,
        Exit(Option<Expression<T, V>>),
        Return,
    }

    #[derive(Clone)]
    pub struct Function<T, V> {
        pub name: String,
        pub parameters: usize,
        pub result: Option<usize>,
        pub types: Vec<T>,
        pub instructions: Vec<Statement<T, V>>,
        pub parameter_names: Vec<String>,
    }

    #[derive(Clone)]
    pub struct Program<T, V> {
        pub functions: Vec<Function<T, V>>,
        pub class_sizes: Vec<usize>,
        pub classes: Vec<Class<T>>,
        pub package_functions: HashMap<String, PackageFunction<T>>,
    }

    #[derive(Clone)]
    pub struct PackageFunction<T> {
        pub wasm_path: String,
        pub command: String,
        pub result: T,
        pub filesystem: u32,
    }

    #[derive(Clone)]
    pub struct Class<T> {
        pub name: String,
        pub fields: Vec<ClassField<T>>,
    }

    #[derive(Clone)]
    pub struct ClassField<T> {
        pub name: String,
        pub ty: T,
        pub public: bool,
    }
}

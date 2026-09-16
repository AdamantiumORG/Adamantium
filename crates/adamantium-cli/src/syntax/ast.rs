use super::diagnostics::Position;
use crate::types::Type;

#[derive(Clone, Copy, Debug)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}
#[derive(Clone, Copy, Debug)]
pub enum LogicalOperator {
    And,
    Or,
}
#[derive(Clone, Copy, Debug)]
pub enum Comparison {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
impl Operator {
    pub(super) fn from_char(c: char) -> Option<Self> {
        match c {
            '+' => Some(Self::Add),
            '-' => Some(Self::Subtract),
            '*' => Some(Self::Multiply),
            '/' => Some(Self::Divide),
            '%' => Some(Self::Remainder),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Expr {
    Integer(i128),
    Decimal(String),
    String(Vec<u8>),
    Bool(bool),
    None,
    EnumVariant(Type, u32),
    Construct(u32, Vec<(String, Expr)>),
    List(Vec<Expr>),
    Index(Box<Expr>, Box<Expr>, Position),
    Annotated(Box<Expr>, Type),
    Cast(Box<Expr>, Type, Position),
    Variable(usize),
    Field(Box<Expr>, String, Position),
    MethodCall(Box<Expr>, String, Vec<Expr>, Position),
    Negate(Box<Expr>),
    Positive(Box<Expr>),
    Not(Box<Expr>),
    Binary(Operator, Box<Expr>, Box<Expr>),
    Logical(LogicalOperator, Box<Expr>, Box<Expr>),
    Compare(Comparison, Box<Expr>, Box<Expr>),
    Call(Call),
    Try(Vec<Statement>),
    Offset(usize),
    Dereference(Box<Expr>, Position),
}
#[derive(Debug)]
pub struct Call {
    pub name: String,
    pub arguments: Vec<Expr>,
    pub position: Position,
}
#[derive(Debug)]
pub enum Statement {
    Noop(Option<String>),
    Assign(usize, Expr),
    Disconnect(usize, usize),
    Remove(usize),
    Clamp(usize, Expr, Expr),
    Print(Expr, bool),
    Call(Call),
    SetField(Expr, String, Expr),
    SetIndex(Expr, Expr, Expr, Position),
    MethodCall(Expr),
    Message(Expr, bool, Position),
    If(Expr, Vec<Statement>, Vec<Statement>),
    While(Expr, Vec<Statement>),
    Until(Expr, Vec<Statement>),
    Loop(Vec<Statement>),
    For(usize, Expr, Expr, Vec<Statement>),
    ForEach(usize, Expr, Vec<Statement>),
    Match(Expr, Vec<(Expr, Vec<Statement>)>, Option<Vec<Statement>>),
    Break,
    Continue,
    Exit(Option<Expr>),
    Return,
}
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub parameters: usize,
    pub optional_parameters: Vec<bool>,
    pub result: Option<usize>,
    pub statements: Vec<Statement>,
    pub positions: Vec<Position>,
    pub types: Vec<Option<Type>>,
    pub bindings: Vec<(String, Position)>,
    pub position: Position,
    pub owner: Option<u32>,
}
#[derive(Clone, Debug)]
pub struct ClassField {
    pub name: String,
    pub ty: Type,
    pub public: bool,
}
#[derive(Clone, Debug)]
pub struct ClassMethod {
    pub name: String,
    pub function: String,
    pub public: bool,
}
#[derive(Clone, Debug)]
pub struct ClassDefinition {
    pub id: u32,
    pub name: String,
    pub fields: Vec<ClassField>,
    pub methods: Vec<ClassMethod>,
    pub traits: Vec<String>,
}
#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
    pub classes: Vec<ClassDefinition>,
    pub enum_variants: Vec<Vec<String>>,
}

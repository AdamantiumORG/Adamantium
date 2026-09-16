use super::diagnostics::Position;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub(super) enum Token {
    Word(String),
    Number(String),
    String(String),
    Symbol(char),
    End,
}

pub(super) type PositionedToken = (Token, Position);
pub(super) type GenericArguments = Vec<Vec<PositionedToken>>;
pub(super) type GenericInstances = HashMap<String, Vec<(String, GenericArguments)>>;

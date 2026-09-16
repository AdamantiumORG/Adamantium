mod ast;
mod diagnostics;
mod lexer;
mod parser;
mod tokens;

pub use ast::*;
pub use parser::{module_dependencies, package_dependencies, parse_modules};

#[cfg(test)]
use crate::types::Type;
#[cfg(test)]
use lexer::lex;
#[cfg(test)]
pub use parser::parse;

#[cfg(test)]
#[path = "../syntax_tests.rs"]
mod tests;

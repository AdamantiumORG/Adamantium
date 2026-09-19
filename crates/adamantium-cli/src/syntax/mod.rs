mod ast;
mod diagnostics;
mod lexer;
mod parser;
mod tokens;

pub use ast::*;
#[cfg(test)]
pub use parser::parse_modules;
pub use parser::{module_dependencies, package_dependencies, parse_modules_with_mode};

pub fn validate_professional(source: &str) -> Result<(), String> {
    use tokens::Token;
    let tokens = lexer::lex(source)?;
    for (index, (token, position)) in tokens.iter().enumerate() {
        if let Token::Word(word) = token
            && matches!(word.as_str(), "int" | "float" | "u")
            && index > 0
            && tokens[index - 1].0 == Token::Symbol(':')
        {
            return Err(position.error(format!(
                "professional mode requires an exact type instead of '{word}'"
            )));
        }
        if !matches!(token, Token::Word(word) if word == "var" || word == "variable") {
            continue;
        }
        let end = tokens[index..]
            .iter()
            .position(|(token, _)| *token == Token::Symbol(';'))
            .map(|offset| index + offset)
            .unwrap_or(tokens.len());
        if !tokens[index..end]
            .iter()
            .any(|(token, _)| *token == Token::Symbol(':'))
        {
            return Err(
                position.error("professional mode requires an explicit concrete variable type")
            );
        }
    }
    Ok(())
}

#[cfg(test)]
use crate::types::Type;
#[cfg(test)]
use lexer::lex;
#[cfg(test)]
pub use parser::parse;

#[cfg(test)]
#[path = "../syntax_tests.rs"]
mod tests;

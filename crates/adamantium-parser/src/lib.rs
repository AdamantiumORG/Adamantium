use adamantium_ast::Identifier;
use adamantium_lexer::{LexError, TokenKind, lex};

#[derive(Debug)]
pub struct ParsedFile {
    pub identifiers: Vec<Identifier>,
}

pub fn parse(source: &str) -> Result<ParsedFile, LexError> {
    Ok(ParsedFile {
        identifiers: lex(source)?
            .into_iter()
            .filter_map(|token| match token.kind {
                TokenKind::Identifier(name) => Some(Identifier::new(name, token.span)),
                _ => None,
            })
            .collect(),
    })
}

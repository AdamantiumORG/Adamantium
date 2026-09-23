use adamantium_ast::Identifier;
use adamantium_lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct ParsedFile {
    pub identifiers: Vec<Identifier>,
}

pub fn parse(tokens: &[Token]) -> ParsedFile {
    ParsedFile {
        identifiers: tokens
            .iter()
            .filter_map(|token| match &token.kind {
                TokenKind::Identifier(name) => Some(Identifier::new(name, token.span)),
                _ => None,
            })
            .collect(),
    }
}

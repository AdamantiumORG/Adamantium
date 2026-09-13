use adamantium_ast::Identifier;
use adamantium_lexer::lex_words;

#[derive(Debug)]
pub struct ParsedFile {
    pub identifiers: Vec<Identifier>,
}

pub fn parse(source: &str) -> ParsedFile {
    ParsedFile {
        identifiers: lex_words(source)
            .into_iter()
            .filter(|token| token.text.chars().next().is_some_and(char::is_alphabetic))
            .map(|token| Identifier::new(token.text, token.span))
            .collect(),
    }
}

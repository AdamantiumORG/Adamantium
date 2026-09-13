use adamantium_ast::Span;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub text: String,
    pub span: Span,
}

pub fn lex_words(source: &str) -> Vec<Token> {
    source
        .lines()
        .enumerate()
        .flat_map(|(line, text)| {
            text.split_whitespace().map(move |word| Token {
                text: word.to_owned(),
                span: Span {
                    line: line + 1,
                    column: text.find(word).unwrap_or(0) + 1,
                },
            })
        })
        .collect()
}

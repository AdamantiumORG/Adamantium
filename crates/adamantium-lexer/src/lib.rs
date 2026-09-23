use adamantium_ast::Span;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier(String),
    Number(String),
    String(String),
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,
    Equals,
    EqualEqual,
    FatArrow,
    Plus,
    Minus,
    Arrow,
    Star,
    Slash,
    Percent,
    Bang,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Pipe,
    LogicalOr,
    Ampersand,
    LogicalAnd,
    Dollar,
    Range,
    DoubleColon,
    LineComment,
    BlockComment,
    Eof,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Keyword {
    And,
    Assert,
    Break,
    Changeable,
    Class,
    Continue,
    Define,
    Else,
    Enum,
    Exit,
    False,
    For,
    Fun,
    If,
    Implements,
    In,
    List,
    Loop,
    Match,
    None,
    Not,
    Offset,
    Or,
    Pack,
    Panic,
    Print,
    Private,
    Public,
    Return,
    Static,
    Then,
    Trait,
    True,
    Until,
    Use,
    Variable,
    Warn,
    While,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn text<'src>(&self, source: &'src str) -> &'src str {
        &source[self.span.start..self.span.end]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl fmt::Display for LexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

impl std::error::Error for LexError {}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token()?;
        let finished = token.kind == TokenKind::Eof;
        tokens.push(token);
        if finished {
            return Ok(tokens);
        }
    }
}

pub struct Lexer<'src> {
    source: &'src str,
    position: usize,
    line: usize,
    column: usize,
    preserve_comments: bool,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            position: 0,
            line: 1,
            column: 1,
            preserve_comments: false,
        }
    }

    pub fn with_comments(source: &'src str) -> Self {
        Self {
            preserve_comments: true,
            ..Self::new(source)
        }
    }

    pub fn next_token(&mut self) -> Result<Token, LexError> {
        loop {
            let Some(character) = self.peek() else {
                return Ok(Token {
                    kind: TokenKind::Eof,
                    span: self.span(),
                });
            };
            let mut span = self.span();
            if character.is_whitespace() {
                self.advance();
                continue;
            }
            if character == '/' && self.peek_next() == Some('/') {
                while self.peek().is_some_and(|value| value != '\n') {
                    self.advance();
                }
                if self.preserve_comments {
                    span.end = self.position;
                    return Ok(Token {
                        kind: TokenKind::LineComment,
                        span,
                    });
                }
                continue;
            }
            if character == '/' && self.peek_next() == Some('*') {
                self.advance();
                self.advance();
                self.block_comment(span)?;
                if self.preserve_comments {
                    span.end = self.position;
                    return Ok(Token {
                        kind: TokenKind::BlockComment,
                        span,
                    });
                }
                continue;
            }
            let kind = if character.is_ascii_alphabetic() || character == '_' {
                self.word()
            } else if character.is_ascii_digit() {
                self.number(span)?
            } else if character == '"' {
                self.advance();
                TokenKind::String(self.string(span)?)
            } else {
                self.operator_or_punctuation().ok_or_else(|| LexError {
                    message: format!("unexpected character {character:?}"),
                    span,
                })?
            };
            span.end = self.position;
            return Ok(Token { kind, span });
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.position..)?.chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        self.source.get(self.position..)?.chars().nth(1)
    }

    fn advance(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.position += character.len_utf8();
        if character == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(character)
    }

    fn span(&self) -> Span {
        Span {
            start: self.position,
            end: self.position,
            line: self.line,
            column: self.column,
        }
    }

    fn take_while(&mut self, predicate: impl Fn(char) -> bool) -> String {
        let mut value = String::new();
        while let Some(character) = self.peek().filter(|value| predicate(*value)) {
            value.push(character);
            self.advance();
        }
        value
    }

    fn word(&mut self) -> TokenKind {
        let value = self.take_while(|value| value.is_ascii_alphanumeric() || value == '_');
        keyword(&value).map_or(TokenKind::Identifier(value), TokenKind::Keyword)
    }

    fn number(&mut self, span: Span) -> Result<TokenKind, LexError> {
        let mut value = self.take_while(|value| value.is_ascii_digit());
        if self.peek() == Some('.') && self.peek_next().is_some_and(|next| next.is_ascii_digit()) {
            value.push(self.advance().unwrap_or('.'));
            value.push_str(&self.take_while(|next| next.is_ascii_digit()));
        }
        if self.peek().is_some_and(|next| matches!(next, 'e' | 'E')) {
            value.push(self.advance().unwrap_or('e'));
            if self.peek().is_some_and(|next| matches!(next, '+' | '-')) {
                value.push(self.advance().unwrap_or('+'));
            }
            if !self.peek().is_some_and(|next| next.is_ascii_digit()) {
                return Err(LexError {
                    message: "expected exponent digits".into(),
                    span,
                });
            }
            value.push_str(&self.take_while(|next| next.is_ascii_digit()));
        }
        Ok(TokenKind::Number(value))
    }

    fn string(&mut self, span: Span) -> Result<String, LexError> {
        let mut value = String::new();
        loop {
            match self.advance() {
                Some('"') => return Ok(value),
                Some('\n' | '\r') => {
                    return Err(LexError {
                        message: "raw newline in string; use \\n".trim_end().into(),
                        span,
                    });
                }
                Some('\\') => {
                    let escaped = self.advance().ok_or_else(|| LexError {
                        message: "unterminated escape".into(),
                        span,
                    })?;
                    value.push(match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '0' => '\0',
                        '\\' => '\\',
                        '"' => '"',
                        _ => {
                            return Err(LexError {
                                message: format!("unsupported string escape \\{escaped}"),
                                span,
                            });
                        }
                    });
                }
                Some(character) => value.push(character),
                None => {
                    return Err(LexError {
                        message: "unterminated string".into(),
                        span,
                    });
                }
            }
        }
    }

    fn block_comment(&mut self, span: Span) -> Result<(), LexError> {
        while let Some(character) = self.advance() {
            if character == '*' && self.peek() == Some('/') {
                self.advance();
                return Ok(());
            }
        }
        Err(LexError {
            message: "unterminated block comment; expected '*/'".into(),
            span,
        })
    }

    fn operator_or_punctuation(&mut self) -> Option<TokenKind> {
        let first = self.peek()?;
        let second = self.peek_next();
        let combined = match (first, second) {
            ('=', Some('=')) => Some(TokenKind::EqualEqual),
            ('=', Some('>')) => Some(TokenKind::FatArrow),
            ('!', Some('=')) => Some(TokenKind::NotEqual),
            ('<', Some('=')) => Some(TokenKind::LessEqual),
            ('>', Some('=')) => Some(TokenKind::GreaterEqual),
            ('-', Some('>')) => Some(TokenKind::Arrow),
            ('|', Some('|')) => Some(TokenKind::LogicalOr),
            ('&', Some('&')) => Some(TokenKind::LogicalAnd),
            ('.', Some('.')) => Some(TokenKind::Range),
            (':', Some(':')) => Some(TokenKind::DoubleColon),
            _ => None,
        };
        if let Some(kind) = combined {
            self.advance();
            self.advance();
            return Some(kind);
        }
        self.advance();
        symbol(first)
    }
}

fn symbol(character: char) -> Option<TokenKind> {
    Some(match character {
        '(' => TokenKind::LeftParen,
        ')' => TokenKind::RightParen,
        '{' => TokenKind::LeftBrace,
        '}' => TokenKind::RightBrace,
        '[' => TokenKind::LeftBracket,
        ']' => TokenKind::RightBracket,
        ';' => TokenKind::Semicolon,
        ':' => TokenKind::Colon,
        ',' => TokenKind::Comma,
        '.' => TokenKind::Dot,
        '=' => TokenKind::Equals,
        '+' => TokenKind::Plus,
        '-' => TokenKind::Minus,
        '*' => TokenKind::Star,
        '/' => TokenKind::Slash,
        '%' => TokenKind::Percent,
        '!' => TokenKind::Bang,
        '<' => TokenKind::Less,
        '>' => TokenKind::Greater,
        '|' => TokenKind::Pipe,
        '&' => TokenKind::Ampersand,
        '$' => TokenKind::Dollar,
        _ => return None,
    })
}

fn keyword(value: &str) -> Option<Keyword> {
    Some(match value {
        "and" => Keyword::And,
        "assert" => Keyword::Assert,
        "break" => Keyword::Break,
        "ch" | "changeable" => Keyword::Changeable,
        "class" => Keyword::Class,
        "continue" => Keyword::Continue,
        "define" => Keyword::Define,
        "else" => Keyword::Else,
        "enum" => Keyword::Enum,
        "exit" => Keyword::Exit,
        "false" => Keyword::False,
        "for" => Keyword::For,
        "fun" => Keyword::Fun,
        "if" => Keyword::If,
        "implements" => Keyword::Implements,
        "in" => Keyword::In,
        "List" => Keyword::List,
        "loop" => Keyword::Loop,
        "match" => Keyword::Match,
        "None" => Keyword::None,
        "not" => Keyword::Not,
        "offset" | "oofset" => Keyword::Offset,
        "or" => Keyword::Or,
        "pack" => Keyword::Pack,
        "panic" => Keyword::Panic,
        "print" => Keyword::Print,
        "priv" => Keyword::Private,
        "pub" => Keyword::Public,
        "return" => Keyword::Return,
        "static" | "stc" => Keyword::Static,
        "then" => Keyword::Then,
        "trait" => Keyword::Trait,
        "true" => Keyword::True,
        "until" => Keyword::Until,
        "use" => Keyword::Use,
        "var" | "variable" => Keyword::Variable,
        "warn" => Keyword::Warn,
        "while" => Keyword::While,
        _ => return None,
    })
}

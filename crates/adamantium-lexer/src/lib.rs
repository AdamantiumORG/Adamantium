use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    pub line: u32,
    pub column: u32,
}

#[derive(Clone, Debug)]
pub struct SourceFile<'src> {
    source: &'src str,
    line_starts: Vec<u32>,
}

impl<'src> SourceFile<'src> {
    pub fn new(source: &'src str) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(
            source
                .bytes()
                .enumerate()
                .filter(|(_, byte)| *byte == b'\n')
                .filter_map(|(offset, _)| u32::try_from(offset + 1).ok()),
        );
        Self {
            source,
            line_starts,
        }
    }

    pub fn text(&self, span: Span) -> Option<&'src str> {
        self.source.get(span.start as usize..span.end as usize)
    }

    pub fn location(&self, offset: u32) -> Option<SourceLocation> {
        if offset as usize > self.source.len() || !self.source.is_char_boundary(offset as usize) {
            return None;
        }
        let line_index = self.line_starts.partition_point(|start| *start <= offset) - 1;
        let line_start = self.line_starts[line_index];
        let column = self
            .source
            .get(line_start as usize..offset as usize)?
            .chars()
            .count()
            + 1;
        Some(SourceLocation {
            line: u32::try_from(line_index + 1).ok()?,
            column: u32::try_from(column).ok()?,
        })
    }

    pub fn span_location(&self, span: Span) -> Option<SourceLocation> {
        self.location(span.start)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier,
    IntLiteral,
    FloatLiteral,
    StringLiteral(String),
    BoolLiteral(bool),
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
    PlusEqual,
    Minus,
    MinusEqual,
    Arrow,
    Star,
    StarEqual,
    Slash,
    SlashEqual,
    Percent,
    PercentEqual,
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
    DotDot,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LexError {
    UnexpectedCharacter { span: Span, character: char },
    InvalidNumber { span: Span, message: String },
    UnterminatedString { span: Span },
    RawNewlineInString { span: Span },
    UnterminatedEscape { span: Span },
    InvalidEscape { span: Span, escape: char },
    UnterminatedComment { span: Span },
}

impl LexError {
    pub fn span(&self) -> Span {
        match self {
            Self::UnexpectedCharacter { span, .. }
            | Self::InvalidNumber { span, .. }
            | Self::UnterminatedString { span }
            | Self::RawNewlineInString { span }
            | Self::UnterminatedEscape { span }
            | Self::InvalidEscape { span, .. }
            | Self::UnterminatedComment { span } => *span,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::UnexpectedCharacter { character, .. } => {
                format!("unexpected character {character:?}")
            }
            Self::InvalidNumber { message, .. } => message.clone(),
            Self::UnterminatedString { .. } => "unterminated string".into(),
            Self::RawNewlineInString { .. } => "raw newline in string; use \\n".into(),
            Self::UnterminatedEscape { .. } => "unterminated string escape".into(),
            Self::InvalidEscape { escape, .. } => {
                format!("unsupported string escape \\{escape}")
            }
            Self::UnterminatedComment { .. } => "unterminated block comment; expected '*/'".into(),
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "bytes {}..{}: {}",
            self.span().start,
            self.span().end,
            self.message()
        )
    }
}

impl std::error::Error for LexError {}

pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut output = lex_recovering(source);
    if output.errors.is_empty() {
        Ok(output.tokens)
    } else {
        Err(output.errors.remove(0))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub errors: Vec<LexError>,
}

pub fn lex_recovering(source: &str) -> LexOutput {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    loop {
        match lexer.next_token() {
            Ok(token) => {
                let finished = token.kind == TokenKind::Eof;
                tokens.push(token);
                if finished {
                    return LexOutput { tokens, errors };
                }
            }
            Err(error) => {
                lexer.recover(&error);
                errors.push(error);
            }
        }
    }
}

pub struct Lexer<'src> {
    source: &'src str,
    position: usize,
    preserve_comments: bool,
    iterator_finished: bool,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            source,
            position: 0,
            preserve_comments: false,
            iterator_finished: false,
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
                    span.end = self.offset();
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
                    span.end = self.offset();
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
                TokenKind::StringLiteral(self.string(span)?)
            } else {
                self.operator_or_punctuation()
                    .ok_or(LexError::UnexpectedCharacter { span, character })?
            };
            span.end = self.offset();
            return Ok(Token { kind, span });
        }
    }

    fn recover(&mut self, error: &LexError) {
        if matches!(error, LexError::InvalidEscape { .. }) {
            while let Some(character) = self.advance() {
                match character {
                    '"' | '\n' | '\r' => break,
                    '\\' => {
                        self.advance();
                    }
                    _ => {}
                }
            }
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
        Some(character)
    }

    fn span(&self) -> Span {
        Span {
            start: self.offset(),
            end: self.offset(),
        }
    }

    fn offset(&self) -> u32 {
        u32::try_from(self.position).unwrap_or(u32::MAX)
    }

    fn word(&mut self) -> TokenKind {
        let start = self.position;
        while self
            .peek()
            .is_some_and(|value| value.is_ascii_alphanumeric() || value == '_')
        {
            self.advance();
        }
        let text = &self.source[start..self.position];
        keyword_kind(text).unwrap_or(TokenKind::Identifier)
    }

    fn number(&mut self, span: Span) -> Result<TokenKind, LexError> {
        let mut float = false;
        while self.peek().is_some_and(|value| value.is_ascii_digit()) {
            self.advance();
        }
        if self.peek() == Some('.') && self.peek_next().is_some_and(|next| next.is_ascii_digit()) {
            float = true;
            self.advance();
            while self.peek().is_some_and(|value| value.is_ascii_digit()) {
                self.advance();
            }
        }
        if self.peek() == Some('.') && self.peek_next().is_some_and(|next| next.is_ascii_digit()) {
            self.advance();
            while self
                .peek()
                .is_some_and(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | '.'))
            {
                self.advance();
            }
            return Err(
                self.number_error(span, "a numeric literal can contain only one decimal point")
            );
        }
        if self.peek().is_some_and(|next| matches!(next, 'e' | 'E')) {
            float = true;
            self.advance();
            if self.peek().is_some_and(|next| matches!(next, '+' | '-')) {
                self.advance();
            }
            if !self.peek().is_some_and(|next| next.is_ascii_digit()) {
                return Err(self.number_error(span, "expected exponent digits"));
            }
            while self.peek().is_some_and(|value| value.is_ascii_digit()) {
                self.advance();
            }
        }
        if self.peek() == Some(':') {
            self.advance();
            let suffix_start = self.position;
            while self
                .peek()
                .is_some_and(|value| value.is_ascii_alphanumeric() || value == '_')
            {
                self.advance();
            }
            let suffix = &self.source[suffix_start..self.position];
            if suffix.is_empty() {
                return Err(self.number_error(span, "expected a type after numeric suffix ':'"));
            }
            if !is_numeric_suffix(suffix) {
                return Err(self.number_error(span, &format!("unknown numeric suffix '{suffix}'")));
            }
        } else if self
            .peek()
            .is_some_and(|value| value.is_ascii_alphabetic() || value == '_')
        {
            while self
                .peek()
                .is_some_and(|value| value.is_ascii_alphanumeric() || value == '_')
            {
                self.advance();
            }
            return Err(
                self.number_error(span, "numeric literals and identifiers must be separated")
            );
        }
        Ok(if float {
            TokenKind::FloatLiteral
        } else {
            TokenKind::IntLiteral
        })
    }

    fn number_error(&self, mut span: Span, message: &str) -> LexError {
        span.end = self.offset();
        LexError::InvalidNumber {
            span,
            message: message.into(),
        }
    }

    fn string(&mut self, span: Span) -> Result<String, LexError> {
        let mut value = String::new();
        loop {
            let character_span = self.span();
            match self.advance() {
                Some('"') => return Ok(value),
                Some('\n' | '\r') => {
                    return Err(LexError::RawNewlineInString {
                        span: self.finished_span(span),
                    });
                }
                Some('\\') => {
                    let escaped = self.advance().ok_or_else(|| LexError::UnterminatedEscape {
                        span: self.finished_span(character_span),
                    })?;
                    value.push(match escaped {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '0' => '\0',
                        '\\' => '\\',
                        '"' => '"',
                        _ => {
                            return Err(LexError::InvalidEscape {
                                span: self.finished_span(character_span),
                                escape: escaped,
                            });
                        }
                    });
                }
                Some(character) => value.push(character),
                None => {
                    return Err(LexError::UnterminatedString {
                        span: self.finished_span(span),
                    });
                }
            }
        }
    }

    fn finished_span(&self, mut span: Span) -> Span {
        span.end = self.offset();
        span
    }

    fn block_comment(&mut self, span: Span) -> Result<(), LexError> {
        while let Some(character) = self.advance() {
            if character == '*' && self.peek() == Some('/') {
                self.advance();
                return Ok(());
            }
        }
        Err(LexError::UnterminatedComment {
            span: self.finished_span(span),
        })
    }

    fn operator_or_punctuation(&mut self) -> Option<TokenKind> {
        let first = self.peek()?;
        let second = self.peek_next();
        if let Some(second) = second
            && let Some(kind) = compound_symbol(first, second)
        {
            self.advance();
            self.advance();
            return Some(kind);
        }
        self.advance();
        symbol(first)
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iterator_finished {
            return None;
        }
        let result = self.next_token();
        if result
            .as_ref()
            .is_ok_and(|token| token.kind == TokenKind::Eof)
        {
            self.iterator_finished = true;
        }
        Some(result)
    }
}

fn compound_symbol(first: char, second: char) -> Option<TokenKind> {
    Some(match (first, second) {
        ('=', '=') => TokenKind::EqualEqual,
        ('=', '>') => TokenKind::FatArrow,
        ('!', '=') => TokenKind::NotEqual,
        ('<', '=') => TokenKind::LessEqual,
        ('>', '=') => TokenKind::GreaterEqual,
        ('-', '>') => TokenKind::Arrow,
        ('+', '=') => TokenKind::PlusEqual,
        ('-', '=') => TokenKind::MinusEqual,
        ('*', '=') => TokenKind::StarEqual,
        ('/', '=') => TokenKind::SlashEqual,
        ('%', '=') => TokenKind::PercentEqual,
        ('|', '|') => TokenKind::LogicalOr,
        ('&', '&') => TokenKind::LogicalAnd,
        ('.', '.') => TokenKind::DotDot,
        (':', ':') => TokenKind::DoubleColon,
        _ => return None,
    })
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

pub fn keyword_kind(text: &str) -> Option<TokenKind> {
    Some(match text {
        "true" => TokenKind::BoolLiteral(true),
        "false" => TokenKind::BoolLiteral(false),
        "and" => TokenKind::Keyword(Keyword::And),
        "assert" => TokenKind::Keyword(Keyword::Assert),
        "break" => TokenKind::Keyword(Keyword::Break),
        "ch" | "changeable" => TokenKind::Keyword(Keyword::Changeable),
        "class" => TokenKind::Keyword(Keyword::Class),
        "continue" => TokenKind::Keyword(Keyword::Continue),
        "define" => TokenKind::Keyword(Keyword::Define),
        "else" => TokenKind::Keyword(Keyword::Else),
        "enum" => TokenKind::Keyword(Keyword::Enum),
        "exit" => TokenKind::Keyword(Keyword::Exit),
        "for" => TokenKind::Keyword(Keyword::For),
        "fun" => TokenKind::Keyword(Keyword::Fun),
        "if" => TokenKind::Keyword(Keyword::If),
        "implements" => TokenKind::Keyword(Keyword::Implements),
        "in" => TokenKind::Keyword(Keyword::In),
        "List" => TokenKind::Keyword(Keyword::List),
        "loop" => TokenKind::Keyword(Keyword::Loop),
        "match" => TokenKind::Keyword(Keyword::Match),
        "None" => TokenKind::Keyword(Keyword::None),
        "not" => TokenKind::Keyword(Keyword::Not),
        "offset" | "oofset" => TokenKind::Keyword(Keyword::Offset),
        "or" => TokenKind::Keyword(Keyword::Or),
        "pack" => TokenKind::Keyword(Keyword::Pack),
        "panic" => TokenKind::Keyword(Keyword::Panic),
        "print" => TokenKind::Keyword(Keyword::Print),
        "priv" => TokenKind::Keyword(Keyword::Private),
        "pub" => TokenKind::Keyword(Keyword::Public),
        "return" => TokenKind::Keyword(Keyword::Return),
        "static" | "stc" => TokenKind::Keyword(Keyword::Static),
        "then" => TokenKind::Keyword(Keyword::Then),
        "trait" => TokenKind::Keyword(Keyword::Trait),
        "until" => TokenKind::Keyword(Keyword::Until),
        "use" => TokenKind::Keyword(Keyword::Use),
        "var" | "variable" => TokenKind::Keyword(Keyword::Variable),
        "warn" => TokenKind::Keyword(Keyword::Warn),
        "while" => TokenKind::Keyword(Keyword::While),
        _ => return None,
    })
}

fn is_numeric_suffix(text: &str) -> bool {
    matches!(
        text,
        "i8" | "i16"
            | "i32"
            | "i64"
            | "int"
            | "u4"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u"
            | "f32"
            | "f64"
            | "f128"
            | "float"
    )
}

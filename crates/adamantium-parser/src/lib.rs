use adamantium_ast::{BinaryOperator, Expression, Identifier, Span, UnaryOperator};
use adamantium_lexer::{SourceFile, Token, TokenKind};

#[derive(Debug)]
pub struct ParsedFile {
    pub identifiers: Vec<Identifier>,
}

pub fn parse(source: &str, tokens: &[Token]) -> ParsedFile {
    let source_file = SourceFile::new(source);
    ParsedFile {
        identifiers: tokens
            .iter()
            .filter_map(|token| match &token.kind {
                TokenKind::Identifier => Some(Identifier::new(
                    source_file.text(token.span).unwrap_or_default(),
                    token.span,
                )),
                _ => None,
            })
            .collect(),
    }
}

pub fn parse_checked(source: &str, tokens: &[Token]) -> Result<ParsedFile, Vec<ParseError>> {
    let errors = tokens
        .windows(2)
        .filter_map(|pair| match (&pair[0].kind, &pair[1].kind) {
            (TokenKind::Equals, TokenKind::Semicolon | TokenKind::Eof) => Some(ParseError {
                message: "expected expression after '='".into(),
                span: pair[1].span,
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    if errors.is_empty() {
        Ok(parse(source, tokens))
    } else {
        Err(errors)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

pub fn parse_expression(source: &str, tokens: &[Token]) -> Result<Expression, ParseError> {
    parse_expression_tokens(source, tokens.iter().cloned())
}

pub fn parse_expression_tokens(
    source: &str,
    tokens: impl IntoIterator<Item = Token>,
) -> Result<Expression, ParseError> {
    let mut parser = Parser {
        source: SourceFile::new(source),
        tokens: TokenStream::new(tokens.into_iter()),
    };
    let expression = parser.expression(0)?;
    if parser.tokens.take(&TokenKind::Semicolon).is_some() {
        // A statement terminator belongs to the surrounding grammar.
    }
    match parser.tokens.peek().map(|token| &token.kind) {
        None | Some(TokenKind::Eof) => Ok(expression),
        Some(_) => Err(parser.error("expected end of expression")),
    }
}

pub struct TokenStream<I>
where
    I: Iterator<Item = Token>,
{
    input: I,
    lookahead: std::collections::VecDeque<Token>,
}

impl<I> TokenStream<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(tokens: I) -> Self {
        Self {
            input: tokens,
            lookahead: std::collections::VecDeque::new(),
        }
    }
    pub fn peek(&mut self) -> Option<&Token> {
        self.peek_n(0)
    }

    pub fn peek_n(&mut self, distance: usize) -> Option<&Token> {
        while self.lookahead.len() <= distance {
            self.lookahead.push_back(self.input.next()?);
        }
        self.lookahead.get(distance)
    }

    pub fn advance(&mut self) -> Option<Token> {
        self.lookahead.pop_front().or_else(|| self.input.next())
    }

    pub fn take(&mut self, expected: &TokenKind) -> Option<Token> {
        if self.peek().is_some_and(|token| &token.kind == expected) {
            self.advance()
        } else {
            None
        }
    }

    pub fn expect(
        &mut self,
        expected: &TokenKind,
        message: impl Into<String>,
    ) -> Result<Token, ParseError> {
        if let Some(token) = self.take(expected) {
            Ok(token)
        } else {
            Err(ParseError {
                message: message.into(),
                span: self.peek().map_or_else(Span::default, |token| token.span),
            })
        }
    }
}

struct Parser<'src, I>
where
    I: Iterator<Item = Token>,
{
    source: SourceFile<'src>,
    tokens: TokenStream<I>,
}

impl<I> Parser<'_, I>
where
    I: Iterator<Item = Token>,
{
    fn expression(&mut self, minimum_precedence: u8) -> Result<Expression, ParseError> {
        let mut left = self.unary()?;
        while let Some((operator, precedence)) = self.binary_operator() {
            if precedence < minimum_precedence {
                break;
            }
            self.tokens.advance();
            let right = self.expression(precedence + 1)?;
            let left_span = left.span();
            let right_span = right.span();
            left = Expression::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
                span: Span {
                    start: left_span.start,
                    end: right_span.end,
                },
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expression, ParseError> {
        if self
            .tokens
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Minus)
        {
            let operator = self.tokens.advance().ok_or_else(|| ParseError {
                message: "expected unary operand".into(),
                span: Span::default(),
            })?;
            let operand = self.unary()?;
            let operand_span = operand.span();
            return Ok(Expression::Unary {
                operator: UnaryOperator::Negate,
                operand: Box::new(operand),
                span: Span {
                    start: operator.span.start,
                    end: operand_span.end,
                },
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.tokens.advance().ok_or_else(|| ParseError {
            message: "expected expression".into(),
            span: Span::default(),
        })?;
        match token.kind {
            TokenKind::Identifier => Ok(Expression::Identifier(Identifier::new(
                self.source.text(token.span).unwrap_or_default(),
                token.span,
            ))),
            TokenKind::IntLiteral | TokenKind::FloatLiteral => Ok(Expression::Number {
                literal: self.source.text(token.span).unwrap_or_default().to_owned(),
                span: token.span,
            }),
            TokenKind::LeftParen => {
                let expression = self.expression(0)?;
                self.tokens
                    .expect(&TokenKind::RightParen, "expected ')' after expression")?;
                Ok(expression)
            }
            _ => Err(ParseError {
                message: "expected identifier, number or parenthesized expression".into(),
                span: token.span,
            }),
        }
    }

    fn binary_operator(&mut self) -> Option<(BinaryOperator, u8)> {
        Some(match &self.tokens.peek()?.kind {
            TokenKind::Plus => (BinaryOperator::Add, 1),
            TokenKind::Minus => (BinaryOperator::Subtract, 1),
            TokenKind::Star => (BinaryOperator::Multiply, 2),
            TokenKind::Slash => (BinaryOperator::Divide, 2),
            TokenKind::Percent => (BinaryOperator::Remainder, 2),
            _ => return None,
        })
    }

    fn error(&mut self, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            span: self
                .tokens
                .peek()
                .map_or_else(Span::default, |token| token.span),
        }
    }
}

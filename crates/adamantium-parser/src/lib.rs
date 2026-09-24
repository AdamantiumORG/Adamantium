use adamantium_ast::{BinaryOperator, Expression, Identifier, Span};
use adamantium_lexer::{Token, TokenKind};

#[derive(Debug)]
pub struct ParsedFile {
    pub identifiers: Vec<Identifier>,
}

pub fn parse(source: &str, tokens: &[Token]) -> ParsedFile {
    ParsedFile {
        identifiers: tokens
            .iter()
            .filter_map(|token| match &token.kind {
                TokenKind::Identifier => Some(Identifier::new(token.text(source), token.span)),
                _ => None,
            })
            .collect(),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

pub fn parse_expression(source: &str, tokens: &[Token]) -> Result<Expression, ParseError> {
    let mut parser = Parser {
        source,
        tokens,
        cursor: 0,
    };
    let expression = parser.expression(0)?;
    if parser.take(&TokenKind::Semicolon) {
        // A statement terminator belongs to the surrounding grammar.
    }
    match parser.peek().map(|token| &token.kind) {
        None | Some(TokenKind::Eof) => Ok(expression),
        Some(_) => Err(parser.error("expected end of expression")),
    }
}

struct Parser<'src, 'tokens> {
    source: &'src str,
    tokens: &'tokens [Token],
    cursor: usize,
}

impl Parser<'_, '_> {
    fn expression(&mut self, minimum_precedence: u8) -> Result<Expression, ParseError> {
        let mut left = self.primary()?;
        while let Some((operator, precedence)) = self.binary_operator() {
            if precedence < minimum_precedence {
                break;
            }
            self.cursor += 1;
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
                    line: left_span.line,
                    column: left_span.column,
                },
            };
        }
        Ok(left)
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        let token = self
            .tokens
            .get(self.cursor)
            .cloned()
            .ok_or_else(|| self.error("expected expression"))?;
        self.cursor += 1;
        match token.kind {
            TokenKind::Identifier => Ok(Expression::Identifier(Identifier::new(
                token.text(self.source),
                token.span,
            ))),
            TokenKind::Number => Ok(Expression::Number {
                literal: token.text(self.source).to_owned(),
                span: token.span,
            }),
            TokenKind::LeftParen => {
                let expression = self.expression(0)?;
                if !self.take(&TokenKind::RightParen) {
                    return Err(self.error("expected ')' after expression"));
                }
                Ok(expression)
            }
            _ => Err(ParseError {
                message: "expected identifier, number or parenthesized expression".into(),
                span: token.span,
            }),
        }
    }

    fn binary_operator(&self) -> Option<(BinaryOperator, u8)> {
        Some(match &self.peek()?.kind {
            TokenKind::Plus => (BinaryOperator::Add, 1),
            TokenKind::Minus => (BinaryOperator::Subtract, 1),
            TokenKind::Star => (BinaryOperator::Multiply, 2),
            TokenKind::Slash => (BinaryOperator::Divide, 2),
            TokenKind::Percent => (BinaryOperator::Remainder, 2),
            _ => return None,
        })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn take(&mut self, expected: &TokenKind) -> bool {
        if self.peek().is_some_and(|token| &token.kind == expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            span: self.peek().map_or_else(Span::default, |token| token.span),
        }
    }
}

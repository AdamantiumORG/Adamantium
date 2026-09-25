use adamantium_ast::{BinaryOperator, Expression, Identifier, Span, UnaryOperator};
use adamantium_lexer::{SourceFile, Token, TokenKind};

#[derive(Debug)]
pub struct ParsedFile {
    pub span: Span,
    pub identifiers: Vec<Identifier>,
}

#[derive(Debug)]
pub struct ParseOutput {
    pub file: ParsedFile,
    pub errors: Vec<ParseError>,
}

pub fn parse(source: &str, tokens: &[Token]) -> ParsedFile {
    let source_file = SourceFile::new(source);
    ParsedFile {
        span: source_file.span(),
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
    let output = parse_recovering(source, tokens);
    if output.errors.is_empty() {
        Ok(output.file)
    } else {
        Err(output.errors)
    }
}

pub fn parse_recovering(source: &str, tokens: &[Token]) -> ParseOutput {
    let mut errors = Vec::new();
    let mut cursor = 0;
    while cursor < tokens.len() {
        if needs_right_operand(&tokens[cursor].kind)
            && tokens
                .get(cursor + 1)
                .is_none_or(|token| is_synchronization_point(&token.kind))
        {
            let span = tokens
                .get(cursor + 1)
                .map_or_else(|| SourceFile::new(source).end_span(), |token| token.span);
            errors.push(ParseError {
                message: format!(
                    "expected expression after '{}'",
                    operator_name(&tokens[cursor].kind)
                ),
                span,
            });
            cursor = synchronize(tokens, cursor + 1);
        } else {
            cursor += 1;
        }
    }
    ParseOutput {
        file: parse(source, tokens),
        errors,
    }
}

fn synchronize(tokens: &[Token], mut cursor: usize) -> usize {
    while let Some(token) = tokens.get(cursor) {
        cursor += 1;
        if is_synchronization_point(&token.kind) {
            break;
        }
    }
    cursor
}

fn is_synchronization_point(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Semicolon | TokenKind::RightBrace | TokenKind::Eof
    )
}

fn needs_right_operand(kind: &TokenKind) -> bool {
    matches!(
        kind,
        TokenKind::Equals
            | TokenKind::PlusEqual
            | TokenKind::MinusEqual
            | TokenKind::StarEqual
            | TokenKind::SlashEqual
            | TokenKind::PercentEqual
            | TokenKind::LogicalOr
            | TokenKind::LogicalAnd
            | TokenKind::EqualEqual
            | TokenKind::NotEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
    )
}

fn operator_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Equals => "=",
        TokenKind::PlusEqual => "+=",
        TokenKind::MinusEqual => "-=",
        TokenKind::StarEqual => "*=",
        TokenKind::SlashEqual => "/=",
        TokenKind::PercentEqual => "%=",
        TokenKind::LogicalOr => "||",
        TokenKind::LogicalAnd => "&&",
        TokenKind::EqualEqual => "==",
        TokenKind::NotEqual => "!=",
        TokenKind::Less => "<",
        TokenKind::LessEqual => "<=",
        TokenKind::Greater => ">",
        TokenKind::GreaterEqual => ">=",
        TokenKind::Plus => "+",
        TokenKind::Minus => "-",
        TokenKind::Star => "*",
        TokenKind::Slash => "/",
        TokenKind::Percent => "%",
        _ => "operator",
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
        let mut left = self.prefix()?;
        loop {
            if POSTFIX_BINDING_POWER >= minimum_precedence
                && let Some(parselet) =
                    postfix_parselet(self.tokens.peek().map(|token| &token.kind))
            {
                left = self.postfix(left, parselet)?;
                continue;
            }
            let Some(parselet) = infix_parselet(self.tokens.peek().map(|token| &token.kind)) else {
                break;
            };
            let InfixParselet {
                operator,
                left_binding_power,
                right_binding_power,
            } = parselet;
            if left_binding_power < minimum_precedence {
                break;
            }
            self.tokens.advance();
            let right = self.expression(right_binding_power)?;
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

    fn prefix(&mut self) -> Result<Expression, ParseError> {
        let unary_operator = prefix_parselet(self.tokens.peek().map(|token| &token.kind));
        if let Some(unary_operator) = unary_operator {
            let operator = self.tokens.advance().ok_or_else(|| ParseError {
                message: "expected prefix operator".into(),
                span: self.source.end_span(),
            })?;
            let operand = self.expression(8)?;
            let operand_span = operand.span();
            return Ok(Expression::Unary {
                operator: unary_operator,
                operand: Box::new(operand),
                span: Span {
                    start: operator.span.start,
                    end: operand_span.end,
                },
            });
        }
        self.primary()
    }

    fn postfix(
        &mut self,
        left: Expression,
        parselet: PostfixParselet,
    ) -> Result<Expression, ParseError> {
        let start = left.span().start;
        match parselet {
            PostfixParselet::Call => {
                self.tokens.advance();
                let mut arguments = Vec::new();
                if let Some(closing) = self.tokens.take(&TokenKind::RightParen) {
                    return Ok(Expression::Call {
                        callee: Box::new(left),
                        arguments,
                        span: Span {
                            start,
                            end: closing.span.end,
                        },
                    });
                }
                loop {
                    arguments.push(self.expression(0)?);
                    if self.tokens.take(&TokenKind::Comma).is_none() {
                        break;
                    }
                }
                let closing = self
                    .tokens
                    .expect(&TokenKind::RightParen, "expected ')' after arguments")?;
                Ok(Expression::Call {
                    callee: Box::new(left),
                    arguments,
                    span: Span {
                        start,
                        end: closing.span.end,
                    },
                })
            }
            PostfixParselet::Index => {
                self.tokens.advance();
                let index = self.expression(0)?;
                let closing = self
                    .tokens
                    .expect(&TokenKind::RightBracket, "expected ']' after index")?;
                Ok(Expression::Index {
                    target: Box::new(left),
                    index: Box::new(index),
                    span: Span {
                        start,
                        end: closing.span.end,
                    },
                })
            }
            PostfixParselet::Member => {
                self.tokens.advance();
                let member = self
                    .tokens
                    .expect(&TokenKind::Identifier, "expected member name after '.'")?;
                Ok(Expression::Member {
                    target: Box::new(left),
                    member: Identifier::new(
                        self.source.text(member.span).unwrap_or_default(),
                        member.span,
                    ),
                    span: Span {
                        start,
                        end: member.span.end,
                    },
                })
            }
        }
    }

    fn primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.tokens.advance().ok_or_else(|| ParseError {
            message: "expected expression".into(),
            span: self.source.end_span(),
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

    fn error(&mut self, message: impl Into<String>) -> ParseError {
        ParseError {
            message: message.into(),
            span: self
                .tokens
                .peek()
                .map_or_else(|| self.source.end_span(), |token| token.span),
        }
    }
}

const POSTFIX_BINDING_POWER: u8 = 9;

#[derive(Clone, Copy)]
enum PostfixParselet {
    Call,
    Index,
    Member,
}

#[derive(Clone, Copy)]
struct InfixParselet {
    operator: BinaryOperator,
    left_binding_power: u8,
    right_binding_power: u8,
}

fn prefix_parselet(kind: Option<&TokenKind>) -> Option<UnaryOperator> {
    match kind? {
        TokenKind::Minus => Some(UnaryOperator::Negate),
        TokenKind::Bang | TokenKind::Keyword(adamantium_lexer::Keyword::Not) => {
            Some(UnaryOperator::Not)
        }
        _ => None,
    }
}

fn postfix_parselet(kind: Option<&TokenKind>) -> Option<PostfixParselet> {
    match kind? {
        TokenKind::LeftParen => Some(PostfixParselet::Call),
        TokenKind::LeftBracket => Some(PostfixParselet::Index),
        TokenKind::Dot => Some(PostfixParselet::Member),
        _ => None,
    }
}

fn infix_parselet(kind: Option<&TokenKind>) -> Option<InfixParselet> {
    let (operator, left_binding_power, right_binding_power) = match kind? {
        TokenKind::Equals => (BinaryOperator::Assign, 1, 1),
        TokenKind::PlusEqual => (BinaryOperator::AddAssign, 1, 1),
        TokenKind::MinusEqual => (BinaryOperator::SubtractAssign, 1, 1),
        TokenKind::StarEqual => (BinaryOperator::MultiplyAssign, 1, 1),
        TokenKind::SlashEqual => (BinaryOperator::DivideAssign, 1, 1),
        TokenKind::PercentEqual => (BinaryOperator::RemainderAssign, 1, 1),
        TokenKind::LogicalOr | TokenKind::Keyword(adamantium_lexer::Keyword::Or) => {
            (BinaryOperator::LogicalOr, 2, 3)
        }
        TokenKind::LogicalAnd | TokenKind::Keyword(adamantium_lexer::Keyword::And) => {
            (BinaryOperator::LogicalAnd, 3, 4)
        }
        TokenKind::EqualEqual => (BinaryOperator::Equal, 4, 5),
        TokenKind::NotEqual => (BinaryOperator::NotEqual, 4, 5),
        TokenKind::Less => (BinaryOperator::Less, 5, 6),
        TokenKind::LessEqual => (BinaryOperator::LessEqual, 5, 6),
        TokenKind::Greater => (BinaryOperator::Greater, 5, 6),
        TokenKind::GreaterEqual => (BinaryOperator::GreaterEqual, 5, 6),
        TokenKind::Plus => (BinaryOperator::Add, 6, 7),
        TokenKind::Minus => (BinaryOperator::Subtract, 6, 7),
        TokenKind::Star => (BinaryOperator::Multiply, 7, 8),
        TokenKind::Slash => (BinaryOperator::Divide, 7, 8),
        TokenKind::Percent => (BinaryOperator::Remainder, 7, 8),
        _ => return None,
    };
    Some(InfixParselet {
        operator,
        left_binding_power,
        right_binding_power,
    })
}

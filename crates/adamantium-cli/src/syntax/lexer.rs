use super::{diagnostics::Position, tokens::Token};

pub(super) fn lex(source: &str) -> Result<Vec<(Token, Position)>, String> {
    let mut chars = source.chars().peekable();
    let (mut line, mut column) = (1, 1);
    let mut tokens = Vec::new();
    while let Some(c) = chars.next() {
        let position = Position { line, column };
        column += 1;
        if c == '\n' {
            line += 1;
            column = 1;
            continue;
        }
        if c.is_whitespace() {
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            while chars.peek().is_some_and(|c| *c != '\n') {
                chars.next();
                column += 1;
            }
            continue;
        }
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            column += 1;
            loop {
                let c = chars
                    .next()
                    .ok_or_else(|| position.error("unterminated block comment; expected '*/'"))?;
                if c == '\n' {
                    line += 1;
                    column = 1;
                } else {
                    column += 1;
                }
                if c == '*' && chars.peek() == Some(&'/') {
                    chars.next();
                    column += 1;
                    break;
                }
            }
            continue;
        }
        let token = if c.is_ascii_alphabetic() || c == '_' {
            let mut word = c.to_string();
            while chars
                .peek()
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
            {
                word.push(chars.next().unwrap());
                column += 1;
            }
            Token::Word(word)
        } else if c.is_ascii_digit() {
            let mut number = c.to_string();
            while chars.peek().is_some_and(char::is_ascii_digit) {
                number.push(chars.next().unwrap());
                column += 1;
            }
            if chars.peek() == Some(&'.')
                && chars.clone().nth(1).is_some_and(|c| c.is_ascii_digit())
            {
                number.push(chars.next().unwrap());
                column += 1;
                while chars.peek().is_some_and(char::is_ascii_digit) {
                    number.push(chars.next().unwrap());
                    column += 1;
                }
            }
            if chars.peek().is_some_and(|c| *c == 'e' || *c == 'E') {
                number.push(chars.next().unwrap());
                column += 1;
                if chars.peek().is_some_and(|c| *c == '+' || *c == '-') {
                    number.push(chars.next().unwrap());
                    column += 1;
                }
                if !chars.peek().is_some_and(char::is_ascii_digit) {
                    return Err(position.error("expected exponent digits"));
                }
                while chars.peek().is_some_and(char::is_ascii_digit) {
                    number.push(chars.next().unwrap());
                    column += 1;
                }
            }
            Token::Number(number)
        } else if c == '"' {
            let mut value = String::new();
            loop {
                let c = chars
                    .next()
                    .ok_or_else(|| position.error("unterminated string"))?;
                column += 1;
                match c {
                    '"' => break,
                    '\n' | '\r' => return Err(position.error("raw newline in string; use \\n")),
                    '\\' => {
                        let escaped = chars
                            .next()
                            .ok_or_else(|| position.error("unterminated escape"))?;
                        column += 1;
                        value.push(match escaped {
                            'n' => '\n',
                            'r' => '\r',
                            't' => '\t',
                            '0' => '\0',
                            '\\' => '\\',
                            '"' => '"',
                            _ => return Err(position.error("unsupported string escape")),
                        });
                    }
                    _ => value.push(c),
                }
            }
            Token::String(value)
        } else if "(){}.;=,:+-*/%[]!<>|&$#".contains(c) {
            Token::Symbol(c)
        } else {
            return Err(position.error(format!("unexpected character {c:?}")));
        };
        tokens.push((token, position));
    }
    tokens.push((Token::End, Position { line, column }));
    Ok(tokens)
}

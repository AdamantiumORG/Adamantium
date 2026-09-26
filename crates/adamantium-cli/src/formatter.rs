//! Deterministic source formatter for Adamantium.
//!
//! The formatter works on a comment-preserving token stream instead of the
//! syntax tree because the parser discards comments and most AST nodes do not
//! keep spans. Formatting a file is a pure function of its text: the same input
//! always produces the same output, and formatting already formatted code
//! leaves it unchanged (idempotent).
//!
//! Canonical layout rules:
//! - 4-space indentation, one statement per line, braces on the opening line.
//! - Every block is expanded: `if x { continue; }` becomes
//!   `if x {` / `    continue;` / `}`.
//! - Blank lines are preserved as written; runs of blank lines are kept intact.
//! - Trailing line comments are moved to their own line at the current indent.
//! - Enum variants are emitted one per line with a trailing comma, matching the
//!   style used by the example project.
//! - Spacing around tokens is canonical: single space between words, tight
//!   parens/brackets, `:`/`.` glued, spaces around binary operators, prefix
//!   markers (`!`, `&`, `$`, unary `+`/`-`, `#[...]`) glued to their operand.
//! - Multi-character operators (`=+`, `=-`, `=*`, `=/`, `==`, `!=`, `<=`,
//!   `>=`, `&&`, `||`, `=>`, `..`) are only recognized when the two symbols are
//!   adjacent in the source, so `x = -5` and `x =*-5` keep their meaning.
//! - Module paths (`pack utils/tools`, `use utils:[module]/tools:fun`) keep
//!   slashes glued to their identifiers, while division is spaced (`17 / 5`).

use std::fmt::Write;

#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Word(String),
    Number(String),
    Str(String),
    Sym(char),
    LineComment(String),
    BlockComment(String),
    Attr(String),
}

#[derive(Clone, Debug)]
struct Tok {
    kind: Kind,
    gap: bool,
    nl: bool,
    blank: usize,
}

fn tokenize(source: &str) -> Result<Vec<Tok>, String> {
    let chars = source.char_indices().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut i = 0usize;
    let mut first = true;
    let (mut gap, mut nl, mut blank) = (false, false, 0usize);
    let mut previous_was_newline = false;
    let (mut line, mut column) = (1usize, 1usize);
    while i < chars.len() {
        let (byte, c) = chars[i];
        match c {
            '\n' => {
                nl = true;
                if previous_was_newline {
                    blank += 1;
                }
                previous_was_newline = true;
                gap = false;
                line += 1;
                column = 1;
                i += 1;
                continue;
            }
            '\r' => {
                gap = true;
                i += 1;
                column += 1;
                continue;
            }
            ' ' | '\t' => {
                gap = true;
                previous_was_newline = false;
                i += 1;
                column += 1;
                continue;
            }
            _ => {}
        }
        let position = format!("{}:{}:", line, column);
        let start = byte;
        let next = {
            if c == '/' && chars.get(i + 1).is_some_and(|(_, n)| *n == '/') {
                let mut j = i + 1;
                while j < chars.len() && chars[j].1 != '\n' {
                    j += 1;
                }
                j
            } else if c == '/' && chars.get(i + 1).is_some_and(|(_, n)| *n == '*') {
                let mut j = i + 2;
                loop {
                    match chars.get(j) {
                        Some(&(_, ch))
                            if ch == '*' && chars.get(j + 1).is_some_and(|(_, n)| *n == '/') =>
                        {
                            j += 2;
                            break;
                        }
                        Some(_) => j += 1,
                        None => {
                            return Err(format!(
                                "{position} unterminated block comment; expected '*/'"
                            ));
                        }
                    }
                }
                j
            } else if c == '#' {
                if chars.get(i + 1).is_none_or(|(_, n)| *n != '[') {
                    return Err(format!("{position} unexpected character '#'"));
                }
                let mut j = i + 2;
                let mut brackets = 1usize;
                loop {
                    let Some(&(_, ch)) = chars.get(j) else {
                        return Err(format!("{position} unterminated attribute; expected ']'"));
                    };
                    if ch == '[' {
                        brackets += 1;
                    } else if ch == ']' {
                        brackets -= 1;
                        if brackets == 0 {
                            j += 1;
                            break;
                        }
                    }
                    j += 1;
                }
                j
            } else if c.is_ascii_alphabetic() || c == '_' {
                let mut j = i + 1;
                while chars
                    .get(j)
                    .is_some_and(|(_, cc)| cc.is_ascii_alphanumeric() || *cc == '_')
                {
                    j += 1;
                }
                j
            } else if c.is_ascii_digit() {
                let mut j = i;
                loop {
                    j += 1;
                    if !chars.get(j).is_some_and(|(_, cc)| cc.is_ascii_digit()) {
                        break;
                    }
                }
                if chars.get(j).is_some_and(|(_, cc)| *cc == '.')
                    && chars.get(j + 1).is_some_and(|(_, cc)| cc.is_ascii_digit())
                {
                    j += 2;
                    while chars.get(j).is_some_and(|(_, cc)| cc.is_ascii_digit()) {
                        j += 1;
                    }
                }
                if chars.get(j).is_some_and(|(_, cc)| *cc == 'e' || *cc == 'E') {
                    j += 1;
                    if chars.get(j).is_some_and(|(_, cc)| *cc == '+' || *cc == '-') {
                        j += 1;
                    }
                    if !chars.get(j).is_some_and(|(_, cc)| cc.is_ascii_digit()) {
                        return Err(format!("{position} expected exponent digits"));
                    }
                    while chars.get(j).is_some_and(|(_, cc)| cc.is_ascii_digit()) {
                        j += 1;
                    }
                }
                j
            } else if c == '"' {
                let mut j = i + 1;
                loop {
                    let Some(&(_, ch)) = chars.get(j) else {
                        return Err(format!("{position} unterminated string"));
                    };
                    j += 1;
                    match ch {
                        '"' => break,
                        '\n' | '\r' => {
                            return Err(format!("{position} raw newline in string; use \\n"));
                        }
                        '\\' => {
                            let Some(&(_, escaped)) = chars.get(j) else {
                                return Err(format!("{position} unterminated escape"));
                            };
                            if !matches!(escaped, 'n' | 'r' | 't' | '0' | '\\' | '"') {
                                return Err(format!("{position} unsupported string escape"));
                            }
                            j += 1;
                        }
                        _ => {}
                    }
                }
                j
            } else if "(){}.;=,:+-*/%[]!<>|&$".contains(c) {
                i + 1
            } else {
                return Err(format!("{position} unexpected character {c:?}"));
            }
        };
        let end = if next == 0 {
            start + c.len_utf8()
        } else {
            chars[next - 1].0 + chars[next - 1].1.len_utf8()
        };
        let raw = &source[start..end];
        let kind = if c == '/' && raw.starts_with("//") {
            Kind::LineComment(raw.to_string())
        } else if c == '/' && raw.starts_with("/*") {
            Kind::BlockComment(raw.to_string())
        } else if c == '#' {
            Kind::Attr(raw.to_string())
        } else if c.is_ascii_alphabetic() || c == '_' {
            Kind::Word(raw.to_string())
        } else if c.is_ascii_digit() {
            Kind::Number(raw.to_string())
        } else if c == '"' {
            Kind::Str(raw.to_string())
        } else {
            Kind::Sym(c)
        };
        let newlines = raw.matches('\n').count();
        line += newlines;
        column = match raw.rfind('\n') {
            Some(lf) => raw[lf + 1..].chars().count() + 1,
            None => column + raw.chars().count(),
        };
        i = next;
        tokens.push(Tok {
            kind,
            gap: !first && gap,
            nl: !first && nl,
            blank: if first { 0 } else { blank },
        });
        first = false;
        gap = false;
        nl = false;
        blank = 0;
        previous_was_newline = false;
    }
    Ok(tokens)
}

struct Layout {
    toks: Vec<Tok>,
    lines: Vec<String>,
    line: String,
    pending_blanks: usize,
    cur_indent: usize,
    depth: usize,
    last: Option<Kind>,
    tight_after: bool,
    statement_started_with: Option<String>,
    statement_start: Option<usize>,
    wrote_anything: bool,
    i: usize,
}

const OPERATOR_PAIRS: &[(&str, char, char)] = &[
    ("==", '=', '='),
    ("=+", '=', '+'),
    ("=-", '=', '-'),
    ("=*", '=', '*'),
    ("=/", '=', '/'),
    ("=>", '=', '>'),
    ("!=", '!', '='),
    ("<=", '<', '='),
    (">=", '>', '='),
    ("&&", '&', '&'),
    ("||", '|', '|'),
    ("..", '.', '.'),
];

fn pair_start(toks: &[Tok], i: usize) -> Option<&'static str> {
    let (cur, next) = (toks.get(i)?, toks.get(i + 1)?);
    if next.nl || next.gap {
        return None;
    }
    let (Kind::Sym(a), Kind::Sym(b)) = (&cur.kind, &next.kind) else {
        return None;
    };
    OPERATOR_PAIRS
        .iter()
        .find(|(_, x, y)| *x == *a && *y == *b)
        .map(|(name, _, _)| *name)
}

fn pair_end(toks: &[Tok], i: usize) -> Option<&'static str> {
    if i == 0 {
        return None;
    }
    let (cur, prev) = (toks.get(i)?, toks.get(i - 1)?);
    if cur.nl || cur.gap {
        return None;
    }
    let (Kind::Sym(a), Kind::Sym(b)) = (&prev.kind, &cur.kind) else {
        return None;
    };
    OPERATOR_PAIRS
        .iter()
        .find(|(_, x, y)| *x == *a && *y == *b)
        .map(|(name, _, _)| *name)
}

impl Layout {
    fn break_line(&mut self) {
        if self.line.is_empty() {
            return;
        }
        for _ in 0..self.pending_blanks {
            self.lines.push(String::new());
        }
        self.pending_blanks = 0;
        let mut full = "    ".repeat(self.cur_indent);
        full.push_str(&self.line);
        self.lines.push(full);
        self.line.clear();
    }

    fn begin_line(&mut self, index: usize) {
        if !self.line.is_empty() {
            return;
        }
        self.cur_indent = self.depth;
        self.statement_started_with = None;
        self.statement_start = None;
        if self.wrote_anything && self.toks[index].blank > 0 {
            self.pending_blanks = self.toks[index].blank;
        }
        let mut k = index;
        while matches!(
            self.toks[k].kind,
            Kind::LineComment(_) | Kind::BlockComment(_)
        ) {
            k += 1;
        }
        self.statement_start = Some(k);
        if let Kind::Word(word) = &self.toks[k].kind {
            self.statement_started_with = Some(word.clone());
        }
    }

    fn is_unary(&self, i: usize) -> bool {
        let (Kind::Sym('-') | Kind::Sym('+')) = &self.toks[i].kind else {
            return false;
        };
        !matches!(
            self.toks.get(i.wrapping_sub(1)).map(|t| &t.kind),
            Some(Kind::Word(_))
                | Some(Kind::Number(_))
                | Some(Kind::Str(_))
                | Some(Kind::Sym(')'))
                | Some(Kind::Sym(']'))
                | Some(Kind::Sym('.'))
        )
    }

    fn is_generic_angle(&self, i: usize) -> bool {
        let Kind::Sym(angle) = &self.toks[i].kind else {
            return false;
        };
        if !matches!(angle, '<' | '>') || self.toks[i].nl || self.toks[i].gap {
            return false;
        }
        let prev = matches!(
            self.toks.get(i.wrapping_sub(1)),
            Some(Tok {
                kind: Kind::Word(_)
                    | Kind::Number(_)
                    | Kind::Sym(')')
                    | Kind::Sym(']')
                    | Kind::Sym('>'),
                ..
            })
        );
        if !prev {
            return false;
        }
        if *angle == '>' {
            return true;
        }
        matches!(
            self.toks.get(i + 1),
            Some(Tok {
                kind: Kind::Word(_) | Kind::Number(_) | Kind::Sym('<'),
                nl: false,
                gap: false,
                ..
            })
        )
    }

    fn is_module_slash(&self, i: usize) -> bool {
        let Kind::Sym('/') = &self.toks[i].kind else {
            return false;
        };
        if self.toks[i].nl || self.toks[i].gap {
            return false;
        }
        let Some(prev) = i.checked_sub(1) else {
            return false;
        };
        let Some(Tok {
            kind: Kind::Word(_),
            ..
        }) = self.toks.get(prev)
        else {
            return false;
        };
        let Some(Tok {
            kind: Kind::Word(_),
            nl: false,
            gap: false,
            ..
        }) = self.toks.get(i + 1)
        else {
            return false;
        };
        if matches!(self.statement_started_with.as_deref(), Some("pack" | "use")) {
            return true;
        }
        let mut j = i + 1;
        while let Some(Tok {
            kind: Kind::Word(_),
            nl: false,
            gap: false,
            ..
        }) = self.toks.get(j)
        {
            match self.toks.get(j + 1) {
                Some(Tok {
                    kind: Kind::Sym(':'),
                    ..
                }) => return true,
                Some(Tok {
                    kind: Kind::Word(_),
                    nl: false,
                    gap: false,
                    ..
                }) => {
                    j += 1;
                }
                Some(Tok {
                    kind: Kind::Sym('/'),
                    nl: false,
                    gap: false,
                    ..
                }) => {
                    j += 2;
                }
                _ => return false,
            }
        }
        false
    }

    fn needs_tight_after(&mut self, i: usize) -> bool {
        if let Some(pair) = pair_start(&self.toks, i).or_else(|| pair_end(&self.toks, i)) {
            return pair == "..";
        }
        match &self.toks[i].kind {
            Kind::Sym('(') | Kind::Sym('[') | Kind::Sym('.') | Kind::Sym(':') => true,
            Kind::Sym('!') | Kind::Sym('&') | Kind::Sym('$') => true,
            Kind::Sym('-') | Kind::Sym('+') => self.is_unary(i),
            Kind::Sym('/') => self.is_module_slash(i),
            Kind::Sym('<') | Kind::Sym('>') => self.is_generic_angle(i),
            _ => false,
        }
    }

    fn separator(&mut self, i: usize) -> &'static str {
        if self.line.is_empty() || self.last.is_none() {
            return "";
        }
        if self.tight_after {
            return "";
        }
        if let Some(pair) = pair_start(&self.toks, i) {
            return if pair == ".." { "" } else { " " };
        }
        if pair_end(&self.toks, i).is_some() {
            return "";
        }
        let cur = &self.toks[i].kind;
        let glue = match cur {
            Kind::Sym(')') | Kind::Sym(']') | Kind::Sym(',') | Kind::Sym('.') | Kind::Sym(':') => {
                true
            }
            Kind::Sym('[') => true,
            Kind::Word(_) | Kind::Number(_) | Kind::Str(_) => false,
            Kind::Sym('(') => matches!(
                self.last,
                Some(
                    Kind::Word(_)
                        | Kind::Number(_)
                        | Kind::Str(_)
                        | Kind::Sym(')')
                        | Kind::Sym(']')
                )
            ),
            Kind::Sym('!') | Kind::Sym('&') | Kind::Sym('$') => {
                matches!(self.last, Some(Kind::Sym('(' | '[' | ',' | ':')))
            }
            Kind::Sym('-') | Kind::Sym('+') => {
                self.is_unary(i) && matches!(self.last, Some(Kind::Sym('(' | '[')))
            }
            Kind::Sym('<') | Kind::Sym('>') => self.is_generic_angle(i),
            Kind::Sym('/') => self.is_module_slash(i),
            _ => false,
        };
        if glue { "" } else { " " }
    }

    fn append_token(&mut self, i: usize) {
        let kind = self.toks[i].kind.clone();
        let separator = self.separator(i);
        self.line.push_str(separator);
        match &kind {
            Kind::Sym(c) => self.line.push(*c),
            Kind::Word(word) | Kind::Number(word) | Kind::Str(word) => self.line.push_str(word),
            _ => unreachable!("content tokens only reach append_token"),
        }
        self.last = Some(kind);
        self.tight_after = self.needs_tight_after(i);
    }

    fn comment_line(&mut self, i: usize, text: String) {
        let blank = self.toks[i].blank;
        let text = match &self.toks[i].kind {
            Kind::LineComment(_) => text.trim_end_matches('\r').to_string(),
            _ => text,
        };
        self.break_line();
        self.begin_line(i);
        if self.wrote_anything && blank > 0 {
            self.pending_blanks = blank;
        }
        self.line.push_str(&text);
        self.break_line();
    }

    fn statement_is_enum(&self) -> bool {
        let Some(start) = self.statement_start else {
            return false;
        };
        start < self.i
            && matches!(self.toks[start].kind, Kind::Word(_))
            && self.toks[start..self.i]
                .iter()
                .any(|t| matches!(&t.kind, Kind::Word(word) if word == "enum"))
    }

    fn matching_brace(&self, open: usize) -> usize {
        let mut depth = 0usize;
        for j in open..self.toks.len() {
            match self.toks[j].kind {
                Kind::Sym('{') => depth += 1,
                Kind::Sym('}') => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return j;
                    }
                }
                _ => {}
            }
        }
        self.toks.len().saturating_sub(1)
    }

    fn enum_variants(&mut self, open: usize) {
        let close = self.matching_brace(open);
        let mut segments = Vec::new();
        let mut start = open + 1;
        let (mut parens, mut brackets, mut braces) = (0usize, 0usize, 0usize);
        let mut has_commas = false;
        for j in open + 1..close {
            match self.toks[j].kind {
                Kind::Sym('(') => parens += 1,
                Kind::Sym(')') => parens = parens.saturating_sub(1),
                Kind::Sym('[') => brackets += 1,
                Kind::Sym(']') => brackets = brackets.saturating_sub(1),
                Kind::Sym('{') => braces += 1,
                Kind::Sym('}') => braces = braces.saturating_sub(1),
                Kind::Sym(',') if parens == 0 && brackets == 0 && braces == 0 => {
                    segments.push(start..j);
                    start = j + 1;
                    has_commas = true;
                }
                _ => {}
            }
        }
        if !has_commas {
            self.i = open + 1;
            return;
        }
        segments.push(start..close);
        self.i = close + 1;
        for segment in segments {
            if segment.is_empty() {
                continue;
            }
            self.begin_line(segment.start);
            self.last = None;
            self.tight_after = false;
            for j in segment {
                self.append_token(j);
            }
            self.line.push(',');
            self.break_line();
        }
        self.depth = self.depth.saturating_sub(1);
        self.cur_indent = self.depth;
        self.line.push('}');
        self.break_line();
        self.statement_started_with = None;
        self.statement_start = None;
    }

    fn run(&mut self) {
        while self.i < self.toks.len() {
            let i = self.i;
            match &self.toks[i].kind {
                Kind::LineComment(_) | Kind::BlockComment(_) | Kind::Attr(_) => {
                    let text = match &self.toks[i].kind {
                        Kind::LineComment(text) => text.clone(),
                        Kind::BlockComment(text) | Kind::Attr(text) => text.clone(),
                        _ => unreachable!(),
                    };
                    self.comment_line(i, text);
                    self.i += 1;
                }
                Kind::Sym(';') => {
                    self.line.push(';');
                    self.break_line();
                    self.statement_started_with = None;
                    self.i += 1;
                }
                Kind::Sym('{') => {
                    self.line.push_str(" {");
                    self.break_line();
                    self.depth += 1;
                    if self.statement_is_enum() {
                        let open = i;
                        self.enum_variants(open);
                    } else {
                        self.i += 1;
                    }
                }
                Kind::Sym('}') => {
                    self.depth = self.depth.saturating_sub(1);
                    self.line.clear();
                    self.cur_indent = self.depth;
                    self.line.push('}');
                    self.i += 1;
                    if matches!(
                        self.toks.get(i + 1),
                        Some(Tok {
                            kind: Kind::Word(word),
                            ..
                        }) if word == "else"
                    ) {
                        self.line.push_str(" else");
                        self.i += 1;
                    } else {
                        self.break_line();
                    }
                    self.statement_started_with = None;
                    self.statement_start = None;
                }
                _ => {
                    self.begin_line(i);
                    self.append_token(i);
                    self.i += 1;
                }
            }
            if !self.lines.is_empty() || !self.line.is_empty() {
                self.wrote_anything = true;
            }
        }
        self.break_line();
    }
}

pub fn format_source(source: &str) -> Result<String, String> {
    let toks = tokenize(source)?;
    let mut layout = Layout {
        toks,
        lines: Vec::new(),
        line: String::new(),
        pending_blanks: 0,
        cur_indent: 0,
        depth: 0,
        last: None,
        tight_after: false,
        statement_started_with: None,
        statement_start: None,
        wrote_anything: false,
        i: 0,
    };
    layout.run();
    if layout.lines.is_empty() {
        return Ok(String::new());
    }
    let mut output = layout.lines.join("\n");
    let _ = writeln!(output);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn format(source: &str) -> String {
        format_source(source).unwrap()
    }

    fn idempotent(source: &str) {
        let once = format(source);
        let twice = format(&once);
        assert_eq!(once, twice, "formatting is not idempotent");
    }

    #[test]
    fn formats_basic_structure_deterministically() {
        let source = "fun main(){var value=1+2;print.newline(value);}";
        let formatted = format(source);
        assert_eq!(
            formatted,
            "fun main() {\n    var value = 1 + 2;\n    print.newline(value);\n}\n"
        );
        assert_eq!(format(&formatted), formatted);
    }

    #[test]
    fn formats_nested_blocks() {
        let source =
            "fun main() { if value >= 10 { print.newline(value); } else { warn(\"no\"); } }";
        let formatted = format(source);
        assert_eq!(
            formatted,
            "fun main() {\n    if value >= 10 {\n        print.newline(value);\n    } else {\n        warn(\"no\");\n    }\n}\n"
        );
        idempotent(source);
    }

    #[test]
    fn preserves_comments_and_moves_trailing_comments() {
        let source = "fun main() {\n    // leading note\n    var x = 5; // trailing note\n    /* block */ print.newline(x);\n}\n";
        let formatted = format(source);
        assert!(formatted.contains("// leading note\n"));
        assert!(formatted.contains("// trailing note\n"));
        assert!(formatted.contains("/* block */\n"));
        assert!(formatted.contains("    print.newline(x);"));
        idempotent(source);
    }

    #[test]
    fn formats_enum_variants_one_per_line() {
        let source = "enum Status{ ready,running,finished }";
        let formatted = format(source);
        assert_eq!(
            formatted,
            "enum Status {\n    ready,\n    running,\n    finished,\n}\n"
        );
        idempotent(source);
    }

    #[test]
    fn keeps_module_path_slashes_glued() {
        let source = "pack utils/tools;\nuse utils:[module_add];\nfun main() { print.newline(utils/tools:module_multiply(3,4)); print.newline(17/5); }\n";
        let formatted = format(source);
        assert!(formatted.contains("pack utils/tools;\n"));
        assert!(formatted.contains("use utils:[module_add];\n"));
        assert!(formatted.contains("utils/tools:module_multiply(3, 4)"));
        assert!(formatted.contains("17 / 5"));
        idempotent(source);
    }

    #[test]
    fn spaces_binary_operators_but_not_prefixes() {
        let source = "var a = -8:i8; var b=!false && (true or false); var c =+ 5;";
        let formatted = format(source);
        assert!(formatted.contains("var a = -8:i8;"));
        assert!(formatted.contains("!false && (true or false);"));
        assert!(formatted.contains("var c =+ 5;"));
        idempotent(source);
    }

    #[test]
    fn distinguishes_compound_assignment_from_signs() {
        let source = "fun main() { var a = 5; a =+ 2; a = -3; }";
        let formatted = format(source);
        assert!(formatted.contains("a =+ 2;"));
        assert!(formatted.contains("a = -3;"));
        idempotent(source);
    }

    #[test]
    fn formats_attributes_and_declarations() {
        let source = "#[test]\nfun addition(){}\n";
        let formatted = format(source);
        assert_eq!(formatted, "#[test]\nfun addition() {\n}\n");
        idempotent(source);
    }

    #[test]
    fn preserves_blank_lines_between_top_level_items() {
        let source = "fun a() {}\n\n\nfun b() {}\n";
        let formatted = format(source);
        assert_eq!(formatted, "fun a() {\n}\n\n\nfun b() {\n}\n");
        idempotent(source);
    }

    #[test]
    fn formats_ranges_generics_and_indexing() {
        let source = "fun main() { for i in 0..3 { print.newline(i); } var x = List[1, 2, 3]; var y = List<int>; }";
        let formatted = format(source);
        assert!(formatted.contains("for i in 0..3 {"));
        assert!(formatted.contains("var x = List[1, 2, 3];"));
        assert!(formatted.contains("var y = List<int>;"));
        idempotent(source);
    }

    #[test]
    fn formatted_example_project_still_parses_and_is_stable() {
        use std::path::Path;
        let code = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("example-project")
            .join("code");
        let modules = adamantium_project::load_modules(&code, |source| {
            crate::syntax::module_dependencies(source).map_err(|e| e.to_string())
        })
        .unwrap();
        assert!(
            crate::syntax::parse_modules(&modules).is_ok(),
            "original example-project must parse"
        );
        let formatted_modules: Vec<(String, String)> = modules
            .iter()
            .map(|(name, source)| (name.clone(), format(source)))
            .collect();
        assert!(
            crate::syntax::parse_modules(&formatted_modules).is_ok(),
            "formatted source must parse"
        );
        for (name, source) in &modules {
            let once = format(source);
            let twice = format(&once);
            assert_eq!(
                once, twice,
                "formatting is not idempotent for module '{name}'"
            );
        }
    }

    #[test]
    fn formats_class_declarations() {
        let source = "class Counter(pub value:int,pub &limit:int){ fun __new__(){ print.newline(\"new\"); } }";
        let formatted = format(source);
        assert!(formatted.contains("class Counter(pub value:int, pub &limit:int) {"));
        assert!(formatted.contains("    fun __new__() {"));
        idempotent(source);
    }

    #[test]
    fn reports_bad_tokens_like_the_lexer() {
        assert!(format_source("fun main() { var x = @; }").is_err());
        assert!(format_source("fun main() { var s = \"unterminated; }").is_err());
    }

    #[test]
    fn empty_strings_are_stable() {
        assert_eq!(format_source("").unwrap(), "");
        assert_eq!(format_source("\n\n").unwrap(), "");
    }
}

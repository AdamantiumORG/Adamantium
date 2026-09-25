use crate::syntax::{Call, Expr, Program, Statement};
use std::collections::{HashMap, HashSet};
use std::path::Path;

const ERROR_SEPARATOR: &str = "\n\u{1e}\n";

pub fn multiple_errors(errors: Vec<String>) -> String {
    errors.join(ERROR_SEPARATOR)
}

pub fn render_errors(errors: &str) -> String {
    errors
        .split(ERROR_SEPARATOR)
        .map(render_error)
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_error(error: &str) -> String {
    if error.starts_with("error[") || error.starts_with("warning[") {
        return error.to_owned();
    }
    let (code, context) = classify(error);
    let mut rendered = format!("error[{code}]: {error}");
    if let Some((path, line, column, message)) = source_location(error) {
        rendered = format!("error[{code}]: {message}\n  --> {path}:{line}:{column}");
        if let Ok(source) = std::fs::read_to_string(path)
            && let Some(text) = source.lines().nth(line.saturating_sub(1))
        {
            rendered.push_str(&format!(
                "\n   |\n{line:>3} | {text}\n   | {}^",
                " ".repeat(column.saturating_sub(1))
            ));
        }
    }
    rendered.push_str(&format!("\n   = context: {context}"));
    if let Some(help) = suggestion(error) {
        rendered.push_str(&format!("\n   = help: {help}"));
    }
    rendered
}

pub fn render_diagnostic(
    diagnostic: &adamantium_diagnostics::Diagnostic,
    path: &Path,
    source: &str,
) -> String {
    let level = match diagnostic.severity {
        adamantium_diagnostics::Severity::Error => "error",
        adamantium_diagnostics::Severity::Warning => "warning",
    };
    let mut rendered = format!("{level}[{}]: {}", diagnostic.code, diagnostic.message);
    render_label(&mut rendered, path, source, &diagnostic.primary, true);
    for label in &diagnostic.secondary {
        render_label(&mut rendered, path, source, label, false);
    }
    rendered.push_str(&format!(
        "\n   = context: {}",
        diagnostic_stage_name(diagnostic.stage)
    ));
    for note in &diagnostic.notes {
        rendered.push_str(&format!("\n   = note: {note}"));
    }
    if let Some(help) = &diagnostic.help {
        rendered.push_str(&format!("\n   = help: {help}"));
    }
    rendered
}

fn diagnostic_stage_name(stage: adamantium_diagnostics::Stage) -> &'static str {
    match stage {
        adamantium_diagnostics::Stage::Lexing => "lexical analysis",
        adamantium_diagnostics::Stage::Parsing => "syntax analysis",
        adamantium_diagnostics::Stage::NameResolution => "name and access resolution",
        adamantium_diagnostics::Stage::TypeChecking => "type checking",
        adamantium_diagnostics::Stage::SemanticAnalysis => "semantic analysis",
        adamantium_diagnostics::Stage::Project => "project configuration",
        adamantium_diagnostics::Stage::Package => "package management",
    }
}

fn render_label(
    rendered: &mut String,
    path: &Path,
    source: &str,
    label: &adamantium_diagnostics::Label,
    primary: bool,
) {
    let Some((line, column, text, width)) = span_line(source, label.span) else {
        return;
    };
    let marker = if primary { '^' } else { '-' };
    rendered.push_str(&format!(
        "\n  --> {}:{line}:{column}\n   |\n{line:>3} | {text}\n   | {}{}",
        path.display(),
        " ".repeat(column.saturating_sub(1)),
        marker.to_string().repeat(width.max(1))
    ));
    if let Some(message) = &label.message {
        rendered.push(' ');
        rendered.push_str(message);
    }
}

fn span_line(source: &str, span: adamantium_ir::Span) -> Option<(usize, usize, &str, usize)> {
    let start = usize::try_from(span.start).ok()?.min(source.len());
    let end = usize::try_from(span.end).ok()?.min(source.len());
    if !source.is_char_boundary(start) || !source.is_char_boundary(end) {
        return None;
    }
    let before = source.get(..start)?;
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = before.rfind('\n').map_or(0, |offset| offset + 1);
    let line_end = source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset);
    let column = source.get(line_start..start)?.chars().count() + 1;
    let width = source.get(start..end)?.chars().count();
    Some((line, column, source.get(line_start..line_end)?, width))
}

fn classify(error: &str) -> (&'static str, &'static str) {
    let lower = error.to_ascii_lowercase();
    if lower.contains("internal compiler error") {
        ("E900", "compiler invariant")
    } else if lower.contains("project.toml") || lower.contains("requirement.toml") {
        ("E400", "project configuration")
    } else if lower.contains("nasm") || lower.contains("linker") || lower.contains("build tools") {
        ("E500", "native toolchain")
    } else if lower.contains("convert") || lower.contains("type") || lower.contains("arithmetic") {
        ("E300", "type checking")
    } else if lower.contains("import")
        || lower.contains("module")
        || lower.contains("declared")
        || lower.contains("private")
        || lower.contains("access")
        || lower.contains("alias")
    {
        ("E200", "name and access resolution")
    } else if lower.contains("expected")
        || lower.contains("unterminated")
        || lower.contains("unexpected")
        || lower.contains("invalid character")
    {
        ("E100", "syntax analysis")
    } else {
        ("E000", "compiler operation")
    }
}

fn suggestion(error: &str) -> Option<&'static str> {
    let lower = error.to_ascii_lowercase();
    if lower.contains("internal compiler error") {
        Some("report the source file and compiler version so this compiler bug can be reproduced")
    } else if lower.contains("expected ';'") {
        Some("add `;` at the end of the statement")
    } else if lower.contains("must declare fun main") {
        Some("add `fun main() { }` to code/main.ad")
    } else if lower.contains("not initialized") {
        Some("assign a value before reading this variable")
    } else if lower.contains("private") {
        Some("make the symbol public or access it from its defining module")
    } else if lower.contains("convert") {
        Some("use a compatible value or an explicit supported `.as(Type)` conversion")
    } else if lower.contains("could not load module") {
        Some("check the `pack` path and the corresponding `.ad` file")
    } else {
        None
    }
}

fn source_location(error: &str) -> Option<(&str, usize, usize, &str)> {
    for (first, _) in error.match_indices(':') {
        let after_first = &error[first + 1..];
        let second_relative = after_first.find(':')?;
        let line = after_first[..second_relative].parse::<usize>().ok();
        let after_second = &after_first[second_relative + 1..];
        let third_relative = after_second.find(':')?;
        let column = after_second[..third_relative].parse::<usize>().ok();
        if let (Some(line), Some(column)) = (line, column) {
            return Some((
                &error[..first],
                line,
                column,
                after_second[third_relative + 1..].trim(),
            ));
        }
    }
    None
}

pub fn warnings(program: &Program) -> Vec<String> {
    let mut warnings = Vec::new();
    let mut graph = HashMap::new();
    for function in &program.functions {
        let mut reads = HashSet::new();
        let mut declared = (0..function.parameters).collect::<HashSet<_>>();
        let mut calls = HashSet::new();
        let mut returned = false;
        let mut warned_unreachable = false;
        for (statement, position) in function.statements.iter().zip(&function.positions) {
            if returned {
                if !warned_unreachable {
                    warnings.push(
                        position.error("warning[W001]: unreachable code after program termination"),
                    );
                    warned_unreachable = true;
                }
                continue;
            }
            match statement {
                Statement::Noop(_) => (),
                Statement::Assign(slot, expr) => {
                    declared.insert(*slot);
                    visit(expr, &mut reads, &mut calls);
                }
                Statement::Disconnect(destination, source) => {
                    declared.insert(*destination);
                    reads.insert(*source);
                }
                Statement::Remove(slot) => {
                    reads.insert(*slot);
                }
                Statement::Clamp(slot, low, high) => {
                    reads.insert(*slot);
                    visit(low, &mut reads, &mut calls);
                    visit(high, &mut reads, &mut calls);
                }
                Statement::Print(expr, _) => visit(expr, &mut reads, &mut calls),
                Statement::Message(expr, _, _) => visit(expr, &mut reads, &mut calls),
                Statement::Assert(value, message, _) => {
                    visit(value, &mut reads, &mut calls);
                    if let Some(message) = message {
                        visit(message, &mut reads, &mut calls);
                    }
                }
                Statement::Call(call) => visit_call(call, &mut reads, &mut calls),
                Statement::SetField(object, _, value) => {
                    visit(object, &mut reads, &mut calls);
                    visit(value, &mut reads, &mut calls);
                }
                Statement::SetIndex(list, index, value, _) => {
                    visit(list, &mut reads, &mut calls);
                    visit(index, &mut reads, &mut calls);
                    visit(value, &mut reads, &mut calls);
                }
                Statement::MethodCall(expr) => visit(expr, &mut reads, &mut calls),
                Statement::If(condition, yes, no) => {
                    visit(condition, &mut reads, &mut calls);
                    for statement in yes.iter().chain(no) {
                        visit_statement(statement, &mut declared, &mut reads, &mut calls);
                    }
                }
                Statement::While(condition, body) | Statement::Until(condition, body) => {
                    visit(condition, &mut reads, &mut calls);
                    for statement in body {
                        visit_statement(statement, &mut declared, &mut reads, &mut calls);
                    }
                }
                Statement::Loop(body) => {
                    for statement in body {
                        visit_statement(statement, &mut declared, &mut reads, &mut calls);
                    }
                }
                Statement::For(slot, start, end, body) => {
                    declared.insert(*slot);
                    visit(start, &mut reads, &mut calls);
                    visit(end, &mut reads, &mut calls);
                    for statement in body {
                        visit_statement(statement, &mut declared, &mut reads, &mut calls);
                    }
                }
                Statement::ForEach(slot, collection, body) => {
                    declared.insert(*slot);
                    visit(collection, &mut reads, &mut calls);
                    for statement in body {
                        visit_statement(statement, &mut declared, &mut reads, &mut calls);
                    }
                }
                Statement::Match(value, arms, fallback) => {
                    visit(value, &mut reads, &mut calls);
                    for (pattern, body) in arms {
                        visit(pattern, &mut reads, &mut calls);
                        for statement in body {
                            visit_statement(statement, &mut declared, &mut reads, &mut calls);
                        }
                    }
                    if let Some(body) = fallback {
                        for statement in body {
                            visit_statement(statement, &mut declared, &mut reads, &mut calls);
                        }
                    }
                }
                Statement::Break | Statement::Continue => (),
                Statement::Return | Statement::Exit(_) => returned = true,
            }
        }
        // A named result is read by both explicit and implicit return.
        if let Some(result) = function.result {
            reads.insert(result);
        }
        for (slot, (name, position)) in function.bindings.iter().enumerate() {
            if declared.contains(&slot)
                && !reads.contains(&slot)
                && !name.starts_with('_')
                && name != "self"
            {
                let kind = if slot < function.parameters {
                    "parameter"
                } else {
                    "variable"
                };
                warnings.push(position.error(format!(
                    "warning[W002]: unused {kind} '{name}' in function '{}'",
                    function.name
                )));
            }
        }
        graph.insert(function.name.as_str(), calls);
    }
    // Follow calls from main: disconnected recursive groups are unused too.
    let mut reached = HashSet::new();
    let mut pending = vec!["main"];
    pending.extend(
        program
            .functions
            .iter()
            .filter(|function| function.owner.is_some())
            .map(|function| function.name.as_str()),
    );
    while let Some(name) = pending.pop() {
        if reached.insert(name)
            && let Some(calls) = graph.get(name)
        {
            pending.extend(calls.iter().map(String::as_str));
        }
    }
    for function in &program.functions {
        if function.owner.is_none()
            && !reached.contains(function.name.as_str())
            && !function.name.starts_with('_')
        {
            warnings.push(function.position.error(format!(
                "warning[W003]: unused function '{}' (not reachable from main)",
                function.name
            )));
        }
    }
    warnings
}

fn visit(expr: &Expr, reads: &mut HashSet<usize>, calls: &mut HashSet<String>) {
    match expr {
        Expr::Variable(slot) => {
            reads.insert(*slot);
        }
        Expr::Call(call) => visit_call(call, reads, calls),
        Expr::Binary(_, a, b) | Expr::Compare(_, a, b) | Expr::Logical(_, a, b) => {
            visit(a, reads, calls);
            visit(b, reads, calls);
        }
        Expr::Negate(expr)
        | Expr::Positive(expr)
        | Expr::Not(expr)
        | Expr::Annotated(expr, _)
        | Expr::Cast(expr, _, _) => visit(expr, reads, calls),
        Expr::Construct(_, fields) => {
            for (_, value) in fields {
                visit(value, reads, calls);
            }
        }
        Expr::List(values) => {
            for value in values {
                visit(value, reads, calls);
            }
        }
        Expr::Index(list, index, _) => {
            visit(list, reads, calls);
            visit(index, reads, calls);
        }
        Expr::Field(object, _, _) => visit(object, reads, calls),
        Expr::MethodCall(object, _, arguments, _) => {
            visit(object, reads, calls);
            for argument in arguments {
                visit(argument, reads, calls);
            }
        }
        Expr::Try(statements) => {
            let mut declared = HashSet::new();
            for statement in statements {
                visit_statement(statement, &mut declared, reads, calls);
            }
        }
        Expr::Offset(slot) => {
            reads.insert(*slot);
        }
        Expr::Dereference(value, _) => visit(value, reads, calls),
        _ => (),
    }
}
fn visit_statement(
    statement: &Statement,
    declared: &mut HashSet<usize>,
    reads: &mut HashSet<usize>,
    calls: &mut HashSet<String>,
) {
    match statement {
        Statement::Noop(_) => (),
        Statement::Assign(slot, expr) => {
            declared.insert(*slot);
            visit(expr, reads, calls);
        }
        Statement::Disconnect(destination, source) => {
            declared.insert(*destination);
            reads.insert(*source);
        }
        Statement::Remove(slot) => {
            reads.insert(*slot);
        }
        Statement::Clamp(slot, low, high) => {
            reads.insert(*slot);
            visit(low, reads, calls);
            visit(high, reads, calls);
        }
        Statement::Print(expr, _)
        | Statement::MethodCall(expr)
        | Statement::Message(expr, _, _) => visit(expr, reads, calls),
        Statement::Assert(value, message, _) => {
            visit(value, reads, calls);
            if let Some(message) = message {
                visit(message, reads, calls);
            }
        }
        Statement::Call(call) => visit_call(call, reads, calls),
        Statement::SetField(object, _, value) => {
            visit(object, reads, calls);
            visit(value, reads, calls);
        }
        Statement::SetIndex(list, index, value, _) => {
            visit(list, reads, calls);
            visit(index, reads, calls);
            visit(value, reads, calls);
        }
        Statement::If(condition, yes, no) => {
            visit(condition, reads, calls);
            for s in yes.iter().chain(no) {
                visit_statement(s, declared, reads, calls);
            }
        }
        Statement::While(condition, body) | Statement::Until(condition, body) => {
            visit(condition, reads, calls);
            for s in body {
                visit_statement(s, declared, reads, calls);
            }
        }
        Statement::Loop(body) => {
            for s in body {
                visit_statement(s, declared, reads, calls);
            }
        }
        Statement::For(slot, start, end, body) => {
            declared.insert(*slot);
            visit(start, reads, calls);
            visit(end, reads, calls);
            for s in body {
                visit_statement(s, declared, reads, calls);
            }
        }
        Statement::ForEach(slot, collection, body) => {
            declared.insert(*slot);
            visit(collection, reads, calls);
            for statement in body {
                visit_statement(statement, declared, reads, calls);
            }
        }
        Statement::Match(value, arms, fallback) => {
            visit(value, reads, calls);
            for (pattern, body) in arms {
                visit(pattern, reads, calls);
                for statement in body {
                    visit_statement(statement, declared, reads, calls);
                }
            }
            if let Some(body) = fallback {
                for statement in body {
                    visit_statement(statement, declared, reads, calls);
                }
            }
        }
        Statement::Exit(code) => {
            if let Some(code) = code {
                visit(code, reads, calls);
            }
        }
        Statement::Break | Statement::Continue | Statement::Return => (),
    }
}
fn visit_call(call: &Call, reads: &mut HashSet<usize>, calls: &mut HashSet<String>) {
    calls.insert(call.name.clone());
    for argument in &call.arguments {
        visit(argument, reads, calls);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_codes_context_highlights_and_suggestions() {
        let path = std::env::temp_dir().join("adamantium-render-diagnostic.ad");
        std::fs::write(&path, "fun main() {\nvar value=1\n}\n").unwrap();
        let rendered = render_errors(&format!(
            "{}:2:12: expected ';', found Symbol('}}')",
            path.display()
        ));
        assert!(rendered.contains("error[E100]"));
        assert!(rendered.contains("2 | var value=1"));
        assert!(rendered.contains('^'));
        assert!(rendered.contains("context: syntax analysis"));
        assert!(rendered.contains("help: add `;`"));
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn renders_structured_primary_secondary_note_and_help_fields() {
        let source = "var value = \"text\";\n";
        let diagnostic = adamantium_diagnostics::Diagnostic::at_stage(
            adamantium_diagnostics::Stage::TypeChecking,
            adamantium_diagnostics::Severity::Error,
            "E300",
            "mismatched types",
            adamantium_ir::Span { start: 12, end: 18 },
        )
        .with_primary_message("expected i32, found string")
        .with_secondary(adamantium_ir::Span { start: 4, end: 9 }, "declared here")
        .with_note("variable types are fixed")
        .with_help("convert the value before assigning it");
        let rendered = render_diagnostic(&diagnostic, Path::new("main.ad"), source);

        assert!(rendered.contains("error[E300]: mismatched types"));
        assert!(rendered.contains("^^^^^^ expected i32, found string"));
        assert!(rendered.contains("----- declared here"));
        assert!(rendered.contains("note: variable types are fixed"));
        assert!(rendered.contains("help: convert the value"));
        assert_eq!(render_errors(&rendered), rendered);
    }
    fn analyze(source: &str) -> Vec<String> {
        let program = crate::syntax::parse(source).unwrap();
        crate::typed::check(&program).unwrap();
        warnings(&program)
    }
    #[test]
    fn ignores_dead_reads_and_calls() {
        let result = analyze(
            "fun main() { entry(); }\nfun entry() r:int {\nvar unused = 10;\nr = 0;\nreturn r;\nprint.newline(unused);\ndead();\n}\nfun dead() r:int { r = 1; }",
        );
        assert!(result.iter().any(|s| s.starts_with("6:1: warning[W001]")));
        assert!(
            result
                .iter()
                .any(|s| s.contains("unused variable 'unused'"))
        );
        assert!(result.iter().any(|s| s.contains("unused function 'dead'")));
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn exit_makes_following_code_unreachable() {
        let result = analyze("fun main() { exit(); print.newline(1); }");
        assert!(
            result
                .iter()
                .any(|warning| warning.contains("warning[W001]"))
        );
    }
    #[test]
    fn follows_calls_and_reports_disconnected_cycles() {
        let result = analyze(
            "fun main() { print.newline(a()); } fun a() r:int { r = b(); } fun b() r:int { r = 1; } fun x() r:int { r = y(); } fun y() r:int { r = x(); }",
        );
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("unused function 'x'"));
        assert!(result[1].contains("unused function 'y'"));
    }
    #[test]
    fn distinguishes_reads_writes_parameters_and_intentional_unused_names() {
        let result = analyze(
            "fun main() { var written = 1; written = 2; var _ignored = 1; print.newline(f(1,2)); } fun f(used:int,unused:int) r:int { var value = used; value =+ 1; value.clamp(0,10); r = value; } fun _reserved() r:None {}",
        );
        assert_eq!(result.len(), 2);
        assert!(result[0].contains("unused variable 'written'"));
        assert!(result[1].contains("unused parameter 'unused'"));
    }
    #[test]
    fn result_variables_and_arguments_are_used() {
        assert!(analyze("fun main() { var a = 1; var result = add(a,2); print.newline(result); } fun add(a:int,b:int) r:int { r = a+b; return r; }").is_empty());
    }
}

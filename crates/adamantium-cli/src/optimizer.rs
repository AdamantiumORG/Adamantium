use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    typed::{Expression, Function, Instruction, Kind, Program},
    types::{self, Type, Value},
};
use adamantium_ir::typed::{ArithmeticOperator, ComparisonOperator, LogicalOperator};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Level {
    O0,
    #[default]
    O1,
    O2,
}

impl Level {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "-O0" => Some(Self::O0),
            "-O1" => Some(Self::O1),
            "-O2" => Some(Self::O2),
            _ => None,
        }
    }
}

pub fn optimize(mut program: Program, entry: &str, level: Level) -> Program {
    if level == Level::O0 {
        return program;
    }
    if level == Level::O2 {
        inline_small_functions(&mut program, entry);
    }
    for function in &mut program.functions {
        optimize_function(function, level);
    }
    if level == Level::O2 {
        remove_dead_functions(&mut program, entry);
    }
    program
}

fn inline_small_functions(program: &mut Program, entry: &str) {
    let candidates = program
        .functions
        .iter()
        .filter(|function| function.name != entry)
        .filter(|function| function.parameters == 0 && function.result.is_none())
        .filter_map(|function| {
            let body = function
                .instructions
                .iter()
                .filter(|instruction| !matches!(instruction, Instruction::Return))
                .cloned()
                .collect::<Vec<_>>();
            (body.len() <= 4 && body.iter().all(inline_safe_statement))
                .then(|| (function.name.clone(), body))
        })
        .collect::<HashMap<_, _>>();
    if candidates.is_empty() {
        return;
    }
    for function in &mut program.functions {
        let mut output = Vec::with_capacity(function.instructions.len());
        for instruction in std::mem::take(&mut function.instructions) {
            if let Instruction::Call(Expression {
                kind: Kind::Call(name, arguments),
                ..
            }) = &instruction
                && arguments.is_empty()
                && let Some(body) = candidates.get(name)
            {
                output.extend(body.iter().cloned());
            } else {
                output.push(instruction);
            }
        }
        function.instructions = output;
    }
}

fn inline_safe_statement(statement: &Instruction) -> bool {
    match statement {
        Instruction::Noop | Instruction::Print(_, _) | Instruction::Message(_, _, _) => true,
        Instruction::Return => true,
        Instruction::Assign(..)
        | Instruction::Disconnect(..)
        | Instruction::Remove(..)
        | Instruction::Clamp(..)
        | Instruction::SetField(..)
        | Instruction::SetIndex(..)
        | Instruction::If(..)
        | Instruction::While(..)
        | Instruction::Until(..)
        | Instruction::Loop(..)
        | Instruction::For(..)
        | Instruction::ForEach(..)
        | Instruction::Match(..)
        | Instruction::Break
        | Instruction::Continue
        | Instruction::Exit(_) => false,
        Instruction::Call(_) => false,
    }
}

fn optimize_function(function: &mut Function, level: Level) {
    let mut constants = HashMap::new();
    optimize_block(&mut function.instructions, &mut constants);
    if level != Level::O2 {
        return;
    }
    let mut used = HashSet::new();
    if let Some(result) = function.result {
        used.insert(result);
    }
    collect_used_block(&function.instructions, &mut used);
    function.instructions = std::mem::take(&mut function.instructions)
        .into_iter()
        .filter_map(|statement| match statement {
            Instruction::Assign(slot, expression) if !used.contains(&slot) => {
                (!pure(&expression)).then_some(Instruction::Call(expression))
            }
            other => Some(other),
        })
        .collect();
}

fn optimize_block(block: &mut Vec<Instruction>, constants: &mut HashMap<usize, (Type, Value)>) {
    let mut output = Vec::with_capacity(block.len());
    for mut statement in std::mem::take(block) {
        optimize_statement(&mut statement, constants);
        match statement {
            Instruction::Noop => {}
            Instruction::If(condition, yes, _no) if boolean(&condition) == Some(true) => {
                output.extend(yes)
            }
            Instruction::If(condition, _yes, no) if boolean(&condition) == Some(false) => {
                output.extend(no)
            }
            Instruction::While(ref condition, _) if boolean(condition) == Some(false) => {}
            Instruction::Until(ref condition, _) if boolean(condition) == Some(true) => {}
            other => {
                let terminal = matches!(
                    other,
                    Instruction::Return
                        | Instruction::Exit(_)
                        | Instruction::Break
                        | Instruction::Continue
                );
                output.push(other);
                if terminal {
                    break;
                }
            }
        }
    }
    *block = output;
}

fn optimize_statement(statement: &mut Instruction, constants: &mut HashMap<usize, (Type, Value)>) {
    match statement {
        Instruction::Assign(slot, expression) => {
            optimize_expression(expression, constants);
            if let Kind::Constant(value) = expression.kind {
                constants.insert(*slot, (expression.ty, value));
            } else {
                constants.remove(slot);
            }
        }
        Instruction::Disconnect(slot, _) => {
            constants.remove(slot);
        }
        Instruction::Remove(expression, _)
        | Instruction::Print(expression, _)
        | Instruction::Call(expression)
        | Instruction::Message(expression, _, _) => optimize_expression(expression, constants),
        Instruction::Clamp(slot, low, high) => {
            optimize_expression(low, constants);
            optimize_expression(high, constants);
            constants.remove(slot);
        }
        Instruction::SetField(object, _, value, _) => {
            optimize_expression(object, constants);
            optimize_expression(value, constants);
            constants.clear();
        }
        Instruction::SetIndex(list, index, value, _) => {
            optimize_expression(list, constants);
            optimize_expression(index, constants);
            optimize_expression(value, constants);
            constants.clear();
        }
        Instruction::If(condition, yes, no) => {
            optimize_expression(condition, constants);
            let mut yes_constants = constants.clone();
            let mut no_constants = constants.clone();
            optimize_block(yes, &mut yes_constants);
            optimize_block(no, &mut no_constants);
            constants.clear();
        }
        Instruction::While(condition, body) | Instruction::Until(condition, body) => {
            // A value known before a loop may be changed by an earlier iteration.
            // Substituting it in the condition or body can turn a terminating loop
            // into an infinite one, so loops start a fresh propagation environment.
            constants.clear();
            optimize_expression(condition, constants);
            let mut inner = HashMap::new();
            optimize_block(body, &mut inner);
        }
        Instruction::Loop(body) => {
            constants.clear();
            let mut inner = HashMap::new();
            optimize_block(body, &mut inner);
        }
        Instruction::For(slot, start, end, body) => {
            optimize_expression(start, constants);
            optimize_expression(end, constants);
            let mut inner = HashMap::new();
            inner.remove(slot);
            optimize_block(body, &mut inner);
            constants.clear();
        }
        Instruction::ForEach(slot, value, body) => {
            optimize_expression(value, constants);
            let mut inner = HashMap::new();
            inner.remove(slot);
            optimize_block(body, &mut inner);
            constants.clear();
        }
        Instruction::Match(value, branches, fallback) => {
            optimize_expression(value, constants);
            for (pattern, body) in branches {
                optimize_expression(pattern, constants);
                let mut inner = constants.clone();
                optimize_block(body, &mut inner);
            }
            if let Some(body) = fallback {
                let mut inner = constants.clone();
                optimize_block(body, &mut inner);
            }
            constants.clear();
        }
        Instruction::Exit(value) => {
            if let Some(value) = value {
                optimize_expression(value, constants);
            }
        }
        Instruction::Noop | Instruction::Break | Instruction::Continue | Instruction::Return => {}
    }
}

fn optimize_expression(expression: &mut Expression, constants: &HashMap<usize, (Type, Value)>) {
    if let Kind::Variable(slot) = expression.kind
        && let Some((ty, value)) = constants.get(&slot)
        && *ty == expression.ty
    {
        expression.kind = Kind::Constant(*value);
        return;
    }
    match &mut expression.kind {
        Kind::Negate(value)
        | Kind::Not(value)
        | Kind::Convert(value)
        | Kind::Dereference(value)
        | Kind::StringLength(value, _)
        | Kind::Field(value, _)
        | Kind::Unwrap(value, _) => optimize_expression(value, constants),
        Kind::Binary(_, left, right)
        | Kind::Compare(_, left, right)
        | Kind::Logical(_, left, right)
        | Kind::Index(left, right, _) => {
            optimize_expression(left, constants);
            optimize_expression(right, constants);
        }
        Kind::Call(_, args) | Kind::Construct(_, args, _) | Kind::List(args) => {
            for arg in args {
                optimize_expression(arg, constants);
            }
        }
        Kind::MethodCall(_, object, args) => {
            optimize_expression(object, constants);
            for arg in args {
                optimize_expression(arg, constants);
            }
        }
        Kind::Try(body) => {
            let mut inner = constants.clone();
            optimize_block(body, &mut inner);
        }
        Kind::Constant(_) | Kind::String(_) | Kind::Variable(_) | Kind::Address(_) => {}
    }
    fold(expression);
}

fn fold(expression: &mut Expression) {
    let folded = match &expression.kind {
        Kind::Negate(value) => constant(value).and_then(|value| {
            types::operation(4, expression.ty, value, Value::default(), Value::default()).ok()
        }),
        Kind::Not(value) => constant(value).map(|value| Value {
            lo: u64::from(value.lo == 0),
            hi: 0,
        }),
        Kind::Binary(operator, left, right) => match (constant(left), constant(right)) {
            (Some(left), Some(right)) => types::operation(
                arithmetic_code(*operator),
                expression.ty,
                left,
                right,
                Value::default(),
            )
            .ok(),
            _ => simplify_arithmetic(*operator, left, right).map(|kind| {
                expression.kind = kind;
                Value::default()
            }),
        },
        Kind::Compare(operator, left, right) => match (constant(left), constant(right)) {
            (Some(left_value), Some(right_value)) if left.ty != Type::String => types::operation(
                comparison_code(*operator),
                left.ty,
                left_value,
                right_value,
                Value::default(),
            )
            .ok(),
            _ => None,
        },
        Kind::Logical(operator, left, right) => match (boolean(left), boolean(right)) {
            (Some(left), Some(right)) => Some(Value {
                lo: match operator {
                    LogicalOperator::And => left && right,
                    LogicalOperator::Or => left || right,
                } as u64,
                hi: 0,
            }),
            _ => None,
        },
        Kind::Convert(value) if !matches!(expression.ty, Type::Optional(_)) => constant(value)
            .and_then(|constant| types::convert(constant, value.ty, expression.ty).ok()),
        _ => None,
    };
    if let Some(value) = folded
        && (!matches!(expression.kind, Kind::Variable(_) | Kind::Constant(_))
            || value != Value::default())
    {
        expression.kind = Kind::Constant(value);
    }
}

fn constant(expression: &Expression) -> Option<Value> {
    if let Kind::Constant(value) = expression.kind {
        Some(value)
    } else {
        None
    }
}
fn boolean(expression: &Expression) -> Option<bool> {
    (expression.ty == Type::Bool)
        .then(|| constant(expression).map(|value| value.lo != 0))
        .flatten()
}
fn zero(expression: &Expression) -> bool {
    constant(expression).is_some_and(|value| value == Value::default())
}

fn simplify_arithmetic(
    operator: ArithmeticOperator,
    left: &Expression,
    right: &Expression,
) -> Option<Kind> {
    match operator {
        ArithmeticOperator::Add | ArithmeticOperator::Subtract if zero(right) => {
            Some(left.kind.clone())
        }
        ArithmeticOperator::Add if zero(left) => Some(right.kind.clone()),
        ArithmeticOperator::Multiply if constant(right).is_some_and(|v| v.lo == 1 && v.hi == 0) => {
            Some(left.kind.clone())
        }
        ArithmeticOperator::Multiply if constant(left).is_some_and(|v| v.lo == 1 && v.hi == 0) => {
            Some(right.kind.clone())
        }
        ArithmeticOperator::Divide if constant(right).is_some_and(|v| v.lo == 1 && v.hi == 0) => {
            Some(left.kind.clone())
        }
        _ => None,
    }
}

fn arithmetic_code(operator: ArithmeticOperator) -> u32 {
    match operator {
        ArithmeticOperator::Add => 0,
        ArithmeticOperator::Subtract => 1,
        ArithmeticOperator::Multiply => 2,
        ArithmeticOperator::Divide => 3,
        ArithmeticOperator::Remainder => 13,
    }
}
fn comparison_code(operator: ComparisonOperator) -> u32 {
    match operator {
        ComparisonOperator::Equal => 7,
        ComparisonOperator::NotEqual => 8,
        ComparisonOperator::Less => 9,
        ComparisonOperator::LessEqual => 10,
        ComparisonOperator::Greater => 11,
        ComparisonOperator::GreaterEqual => 12,
    }
}

fn pure(expression: &Expression) -> bool {
    match &expression.kind {
        Kind::Constant(_) | Kind::String(_) | Kind::Variable(_) | Kind::Address(_) => true,
        Kind::Not(v) => pure(v),
        Kind::Logical(_, a, b) => pure(a) && pure(b),
        Kind::Negate(_)
        | Kind::Convert(_)
        | Kind::StringLength(_, _)
        | Kind::Field(_, _)
        | Kind::Binary(_, _, _)
        | Kind::Compare(_, _, _)
        | Kind::List(_) => false,
        Kind::Call(_, _)
        | Kind::Construct(_, _, _)
        | Kind::MethodCall(_, _, _)
        | Kind::Try(_)
        | Kind::Unwrap(_, _)
        | Kind::Index(_, _, _)
        | Kind::Dereference(_) => false,
    }
}

fn collect_used_expression(expression: &Expression, used: &mut HashSet<usize>) {
    match &expression.kind {
        Kind::Variable(slot) | Kind::Address(slot) => {
            used.insert(*slot);
        }
        Kind::Negate(v)
        | Kind::Not(v)
        | Kind::Convert(v)
        | Kind::Unwrap(v, _)
        | Kind::StringLength(v, _)
        | Kind::Field(v, _)
        | Kind::Dereference(v) => collect_used_expression(v, used),
        Kind::Binary(_, a, b)
        | Kind::Compare(_, a, b)
        | Kind::Logical(_, a, b)
        | Kind::Index(a, b, _) => {
            collect_used_expression(a, used);
            collect_used_expression(b, used);
        }
        Kind::Call(_, args) | Kind::Construct(_, args, _) | Kind::List(args) => {
            for value in args {
                collect_used_expression(value, used);
            }
        }
        Kind::MethodCall(_, object, args) => {
            collect_used_expression(object, used);
            for value in args {
                collect_used_expression(value, used);
            }
        }
        Kind::Try(body) => collect_used_block(body, used),
        Kind::Constant(_) | Kind::String(_) => {}
    }
}

fn collect_used_block(block: &[Instruction], used: &mut HashSet<usize>) {
    for statement in block {
        collect_used_statement(statement, used);
    }
}
fn collect_used_statement(statement: &Instruction, used: &mut HashSet<usize>) {
    match statement {
        Instruction::Assign(_, v)
        | Instruction::Remove(v, _)
        | Instruction::Print(v, _)
        | Instruction::Call(v)
        | Instruction::Message(v, _, _) => collect_used_expression(v, used),
        Instruction::Disconnect(a, b) => {
            used.insert(*a);
            used.insert(*b);
        }
        Instruction::Clamp(slot, a, b) => {
            used.insert(*slot);
            collect_used_expression(a, used);
            collect_used_expression(b, used);
        }
        Instruction::SetField(a, _, b, _) => {
            collect_used_expression(a, used);
            collect_used_expression(b, used);
        }
        Instruction::SetIndex(a, b, c, _) => {
            collect_used_expression(a, used);
            collect_used_expression(b, used);
            collect_used_expression(c, used);
        }
        Instruction::If(v, a, b) => {
            collect_used_expression(v, used);
            collect_used_block(a, used);
            collect_used_block(b, used);
        }
        Instruction::While(v, b) | Instruction::Until(v, b) | Instruction::ForEach(_, v, b) => {
            collect_used_expression(v, used);
            collect_used_block(b, used);
        }
        Instruction::Loop(b) => collect_used_block(b, used),
        Instruction::For(_, a, b, c) => {
            collect_used_expression(a, used);
            collect_used_expression(b, used);
            collect_used_block(c, used);
        }
        Instruction::Match(v, branches, fallback) => {
            collect_used_expression(v, used);
            for (p, b) in branches {
                collect_used_expression(p, used);
                collect_used_block(b, used);
            }
            if let Some(b) = fallback {
                collect_used_block(b, used);
            }
        }
        Instruction::Exit(Some(v)) => collect_used_expression(v, used),
        Instruction::Noop
        | Instruction::Break
        | Instruction::Continue
        | Instruction::Exit(None)
        | Instruction::Return => {}
    }
}

fn remove_dead_functions(program: &mut Program, entry: &str) {
    let by_name = program
        .functions
        .iter()
        .enumerate()
        .map(|(i, f)| (f.name.clone(), i))
        .collect::<HashMap<_, _>>();
    let mut live = HashSet::new();
    let mut queue = VecDeque::from([entry.to_string()]);
    while let Some(name) = queue.pop_front() {
        if !live.insert(name.clone()) {
            continue;
        }
        if let Some(index) = by_name.get(&name) {
            collect_calls_block(&program.functions[*index].instructions, &mut queue);
        }
    }
    program
        .functions
        .retain(|function| live.contains(&function.name));
}

fn collect_calls_expression(expression: &Expression, calls: &mut VecDeque<String>) {
    match &expression.kind {
        Kind::Call(name, args) | Kind::Construct(_, args, name) => {
            calls.push_back(name.clone());
            for a in args {
                collect_calls_expression(a, calls);
            }
        }
        Kind::MethodCall(name, o, args) => {
            calls.push_back(name.clone());
            collect_calls_expression(o, calls);
            for a in args {
                collect_calls_expression(a, calls);
            }
        }
        Kind::Negate(v)
        | Kind::Not(v)
        | Kind::Convert(v)
        | Kind::Unwrap(v, _)
        | Kind::StringLength(v, _)
        | Kind::Field(v, _)
        | Kind::Dereference(v) => collect_calls_expression(v, calls),
        Kind::Binary(_, a, b)
        | Kind::Compare(_, a, b)
        | Kind::Logical(_, a, b)
        | Kind::Index(a, b, _) => {
            collect_calls_expression(a, calls);
            collect_calls_expression(b, calls);
        }
        Kind::List(values) => {
            for v in values {
                collect_calls_expression(v, calls)
            }
        }
        Kind::Try(body) => collect_calls_block(body, calls),
        _ => {}
    }
}
fn collect_calls_block(block: &[Instruction], calls: &mut VecDeque<String>) {
    for s in block {
        match s {
            Instruction::Assign(_, v)
            | Instruction::Print(v, _)
            | Instruction::Call(v)
            | Instruction::Message(v, _, _) => collect_calls_expression(v, calls),
            Instruction::Remove(v, hook) => {
                collect_calls_expression(v, calls);
                if let Some(hook) = hook {
                    calls.push_back(hook.clone());
                }
            }
            Instruction::Clamp(_, a, b) => {
                collect_calls_expression(a, calls);
                collect_calls_expression(b, calls)
            }
            Instruction::SetField(a, _, b, hook) => {
                collect_calls_expression(a, calls);
                collect_calls_expression(b, calls);
                if let Some(hook) = hook {
                    calls.push_back(hook.clone());
                }
            }
            Instruction::SetIndex(a, b, c, _) => {
                collect_calls_expression(a, calls);
                collect_calls_expression(b, calls);
                collect_calls_expression(c, calls)
            }
            Instruction::If(v, a, b) => {
                collect_calls_expression(v, calls);
                collect_calls_block(a, calls);
                collect_calls_block(b, calls)
            }
            Instruction::While(v, b) | Instruction::Until(v, b) | Instruction::ForEach(_, v, b) => {
                collect_calls_expression(v, calls);
                collect_calls_block(b, calls)
            }
            Instruction::Loop(b) => collect_calls_block(b, calls),
            Instruction::For(_, a, b, c) => {
                collect_calls_expression(a, calls);
                collect_calls_expression(b, calls);
                collect_calls_block(c, calls)
            }
            Instruction::Match(v, branches, fallback) => {
                collect_calls_expression(v, calls);
                for (p, b) in branches {
                    collect_calls_expression(p, calls);
                    collect_calls_block(b, calls)
                }
                if let Some(b) = fallback {
                    collect_calls_block(b, calls)
                }
            }
            Instruction::Exit(Some(v)) => collect_calls_expression(v, calls),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(number: i128) -> Expression {
        Expression {
            ty: Type::I32,
            kind: Kind::Constant(types::integer(number, Type::I32).unwrap()),
        }
    }

    fn function(name: &str, instructions: Vec<Instruction>) -> Function {
        Function {
            name: name.into(),
            parameters: 0,
            result: None,
            types: vec![Type::I32],
            instructions,
            parameter_names: Vec::new(),
        }
    }

    #[test]
    fn folds_propagates_and_removes_dead_ir() {
        let expression = Expression {
            ty: Type::I32,
            kind: Kind::Binary(
                ArithmeticOperator::Add,
                Box::new(value(2)),
                Box::new(value(3)),
            ),
        };
        let mut program = Program {
            functions: vec![
                function(
                    "main",
                    vec![
                        Instruction::Assign(0, expression),
                        Instruction::Print(
                            Expression {
                                ty: Type::I32,
                                kind: Kind::Variable(0),
                            },
                            true,
                        ),
                        Instruction::Return,
                        Instruction::Print(value(99), true),
                    ],
                ),
                function("unused", vec![Instruction::Return]),
            ],
            class_sizes: Vec::new(),
            classes: Vec::new(),
            package_functions: HashMap::new(),
        };
        program = optimize(program, "main", Level::O2);
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].instructions.len(), 2);
        let Instruction::Print(expression, _) = &program.functions[0].instructions[0] else {
            panic!()
        };
        assert_eq!(constant(expression).unwrap().integer(Type::I32), 5);
    }

    #[test]
    fn retains_transitively_called_functions() {
        let call = Expression {
            ty: Type::None,
            kind: Kind::Call("helper".into(), Vec::new()),
        };
        let program = Program {
            functions: vec![
                function("main", vec![Instruction::Call(call)]),
                function("helper", vec![Instruction::Exit(None)]),
                function("dead", vec![Instruction::Return]),
            ],
            class_sizes: Vec::new(),
            classes: Vec::new(),
            package_functions: HashMap::new(),
        };
        let program = optimize(program, "main", Level::O2);
        assert_eq!(
            program
                .functions
                .iter()
                .map(|f| f.name.as_str())
                .collect::<Vec<_>>(),
            ["main", "helper"]
        );
    }

    #[test]
    fn inlines_small_stateless_functions_at_o2() {
        let call = Expression {
            ty: Type::None,
            kind: Kind::Call("announce".into(), Vec::new()),
        };
        let program = Program {
            functions: vec![
                function("main", vec![Instruction::Call(call)]),
                function("announce", vec![Instruction::Print(value(7), true)]),
            ],
            class_sizes: Vec::new(),
            classes: Vec::new(),
            package_functions: HashMap::new(),
        };
        let program = optimize(program, "main", Level::O2);
        assert_eq!(program.functions.len(), 1);
        assert!(matches!(
            program.functions[0].instructions.as_slice(),
            [Instruction::Print(
                Expression {
                    kind: Kind::Constant(_),
                    ..
                },
                true
            )]
        ));
    }

    #[test]
    fn preserves_unused_expressions_that_can_raise_runtime_errors() {
        let maximum = Expression {
            ty: Type::I8,
            kind: Kind::Constant(types::integer(127, Type::I8).unwrap()),
        };
        let one = Expression {
            ty: Type::I8,
            kind: Kind::Constant(types::integer(1, Type::I8).unwrap()),
        };
        let overflowing = Expression {
            ty: Type::I8,
            kind: Kind::Binary(ArithmeticOperator::Add, Box::new(maximum), Box::new(one)),
        };
        let mut function = function("main", vec![Instruction::Assign(0, overflowing)]);
        function.types[0] = Type::I8;
        let program = Program {
            functions: vec![function],
            class_sizes: Vec::new(),
            classes: Vec::new(),
            package_functions: HashMap::new(),
        };
        let program = optimize(program, "main", Level::O2);
        assert!(matches!(
            program.functions[0].instructions[0],
            Instruction::Call(Expression {
                kind: Kind::Binary(..),
                ..
            })
        ));
    }

    #[test]
    fn keeps_wide_optional_allocation_in_the_generated_program() {
        let source = Expression {
            ty: Type::F128,
            kind: Kind::Constant(types::literal("1.25", Type::F128).unwrap()),
        };
        let mut expression = Expression {
            ty: Type::Optional(Type::F128.id()),
            kind: Kind::Convert(Box::new(source)),
        };
        optimize_expression(&mut expression, &HashMap::new());
        assert!(matches!(expression.kind, Kind::Convert(_)));
    }

    #[test]
    fn does_not_propagate_pre_loop_constants_into_repeated_iterations() {
        let variable = || Expression {
            ty: Type::I32,
            kind: Kind::Variable(0),
        };
        let increment = Expression {
            ty: Type::I32,
            kind: Kind::Binary(
                ArithmeticOperator::Add,
                Box::new(variable()),
                Box::new(value(1)),
            ),
        };
        let stop = Expression {
            ty: Type::Bool,
            kind: Kind::Compare(
                ComparisonOperator::Equal,
                Box::new(variable()),
                Box::new(value(2)),
            ),
        };
        let program = Program {
            functions: vec![function(
                "main",
                vec![
                    Instruction::Assign(0, value(0)),
                    Instruction::Loop(vec![
                        Instruction::Assign(0, increment),
                        Instruction::If(stop, vec![Instruction::Break], Vec::new()),
                    ]),
                ],
            )],
            class_sizes: Vec::new(),
            classes: Vec::new(),
            package_functions: HashMap::new(),
        };

        let program = optimize(program, "main", Level::O1);
        let Instruction::Loop(body) = &program.functions[0].instructions[1] else {
            panic!("loop expected");
        };
        assert!(matches!(
            body.as_slice(),
            [
                Instruction::Assign(
                    0,
                    Expression {
                        kind: Kind::Binary(_, _, _),
                        ..
                    }
                ),
                Instruction::If(
                    Expression {
                        kind: Kind::Compare(_, _, _),
                        ..
                    },
                    _,
                    _
                )
            ]
        ));
    }
}

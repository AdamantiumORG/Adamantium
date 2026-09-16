use std::collections::HashMap;

use adamantium_ir::typed::{
    ArithmeticOperator, Expression, ExpressionKind, Function, Program, Statement,
};
use adamantium_types::PrimitiveType;

#[test]
fn creates_typed_integer_instruction() {
    assert!(matches!(
        adamantium_ir::integer(4),
        adamantium_ir::Instruction::Constant { value: 4, .. }
    ));
}

#[test]
fn represents_a_typed_program_without_backend_details() {
    let expression = Expression {
        ty: PrimitiveType::Int,
        kind: ExpressionKind::Binary(
            ArithmeticOperator::Add,
            Box::new(Expression {
                ty: PrimitiveType::Int,
                kind: ExpressionKind::Constant(20_i64),
            }),
            Box::new(Expression {
                ty: PrimitiveType::Int,
                kind: ExpressionKind::Constant(22_i64),
            }),
        ),
    };
    let program = Program {
        functions: vec![Function {
            name: "main".into(),
            parameters: 0,
            result: None,
            types: vec![PrimitiveType::Int],
            instructions: vec![Statement::Assign(0, expression), Statement::Return],
            parameter_names: Vec::new(),
        }],
        class_sizes: Vec::new(),
        classes: Vec::new(),
        package_functions: HashMap::new(),
    };

    assert_eq!(program.functions[0].name, "main");
    assert!(matches!(
        program.functions[0].instructions[0],
        Statement::Assign(
            0,
            Expression {
                kind: ExpressionKind::Binary(ArithmeticOperator::Add, _, _),
                ..
            }
        )
    ));
}

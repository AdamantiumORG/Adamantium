use adamantium_hir::DefId;
use adamantium_lexer::Span;
use adamantium_types::TypeId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Instruction {
    pub kind: InstructionKind,
    pub ty: TypeId,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InstructionKind {
    Define(DefId),
    Return,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Program {
    pub instructions: Vec<Instruction>,
}

pub fn lower(hir: &adamantium_hir::Program, none: TypeId) -> Program {
    let mut instructions = hir
        .definitions
        .iter()
        .map(|definition| Instruction {
            kind: InstructionKind::Define(definition.id),
            ty: definition.ty,
            span: definition.span,
        })
        .collect::<Vec<_>>();
    instructions.push(Instruction {
        kind: InstructionKind::Return,
        ty: none,
        span: Span::default(),
    });
    Program { instructions }
}

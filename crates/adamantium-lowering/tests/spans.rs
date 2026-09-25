use adamantium_hir::{DefId, Definition, SymbolId};
use adamantium_ir::Span;
use adamantium_types::{Type, TypeInterner, TypeKind};

#[test]
fn preserves_spans_across_hir_mir_and_ir() {
    let mut types = TypeInterner::default();
    let integer = types.intern(TypeKind::Primitive(Type::I32));
    let none = types.intern(TypeKind::Primitive(Type::None));
    let program_span = Span { start: 0, end: 24 };
    let definition_span = Span { start: 4, end: 13 };
    let hir = adamantium_hir::Program {
        span: program_span,
        definitions: vec![Definition {
            id: DefId(0),
            symbol: SymbolId(0),
            span: definition_span,
            ty: integer,
        }],
    };

    let mir = adamantium_lowering::hir_to_mir(&hir, none);
    assert_eq!(mir.span, program_span);
    assert_eq!(mir.instructions[0].span, definition_span);
    assert_eq!(mir.instructions[1].span, Span::empty_at(program_span.end));

    let ir = adamantium_lowering::mir_to_ir(&mir);
    assert_eq!(ir.span, program_span);
    assert_eq!(ir.instructions[0].span, Span::empty_at(program_span.end));
}

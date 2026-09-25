use adamantium_hir::{DefId, Definition};
use adamantium_lexer::Span;
use adamantium_types::{Type, TypeInterner, TypeKind};

#[test]
fn lowering_preserves_definition_spans_and_type_ids() {
    let mut types = TypeInterner::default();
    let integer = types.intern(TypeKind::Primitive(Type::I32));
    let none = types.intern(TypeKind::Primitive(Type::None));
    let source_span = Span { start: 4, end: 8 };
    let hir = adamantium_hir::Program {
        span: Span { start: 0, end: 12 },
        definitions: vec![Definition {
            id: DefId(0),
            symbol: adamantium_hir::SymbolId(0),
            span: source_span,
            ty: integer,
        }],
    };
    let mir = adamantium_mir::lower(&hir, none);
    assert_eq!(mir.instructions[0].span, source_span);
    assert_eq!(mir.instructions[0].ty, integer);
    assert_eq!(mir.instructions[1].ty, none);
    assert_eq!(mir.span, hir.span);
    assert_eq!(mir.instructions[1].span, Span::empty_at(hir.span.end));
}

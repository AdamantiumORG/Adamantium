use adamantium_hir::SymbolInterner;

#[test]
fn interns_each_symbol_once() {
    let mut symbols = SymbolInterner::default();
    let first = symbols.intern("value");
    let second = symbols.intern("value");
    assert_eq!(first, second);
    assert_eq!(symbols.resolve(first), Some("value"));
}

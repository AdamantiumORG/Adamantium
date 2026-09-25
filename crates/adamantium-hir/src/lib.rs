use adamantium_lexer::Span;
use adamantium_types::TypeId;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SymbolId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DefId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct LocalId(pub u32);

#[derive(Clone, Debug, Default)]
pub struct SymbolInterner {
    symbols: Vec<String>,
    ids: HashMap<String, SymbolId>,
}

impl SymbolInterner {
    pub fn intern(&mut self, text: &str) -> SymbolId {
        if let Some(id) = self.ids.get(text) {
            return *id;
        }
        let id = SymbolId(u32::try_from(self.symbols.len()).expect("too many symbols"));
        self.symbols.push(text.to_owned());
        self.ids.insert(text.to_owned(), id);
        id
    }

    pub fn resolve(&self, id: SymbolId) -> Option<&str> {
        self.symbols.get(id.0 as usize).map(String::as_str)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Definition {
    pub id: DefId,
    pub symbol: SymbolId,
    pub span: Span,
    pub ty: TypeId,
}

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub definitions: Vec<Definition>,
}

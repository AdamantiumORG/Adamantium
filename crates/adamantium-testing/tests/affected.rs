use std::collections::{BTreeSet, HashMap};

#[test]
fn includes_transitive_downstream_packages() {
    let changed = BTreeSet::from(["lexer".to_owned()]);
    let graph = HashMap::from([
        ("parser".to_owned(), BTreeSet::from(["lexer".to_owned()])),
        ("compiler".to_owned(), BTreeSet::from(["parser".to_owned()])),
        ("linker".to_owned(), BTreeSet::new()),
    ]);
    assert_eq!(
        adamantium_testing::affected_packages(&changed, &graph),
        BTreeSet::from([
            "compiler".to_owned(),
            "lexer".to_owned(),
            "parser".to_owned()
        ])
    );
}

#[test]
fn testing_crate_uses_public_workspace_boundaries() {
    assert!(adamantium_testing::architecture_is_connected());
}

#[test]
fn independent_packages_are_not_selected() {
    let changed = BTreeSet::from(["linker".to_owned()]);
    let graph = HashMap::from([
        ("compiler".to_owned(), BTreeSet::from(["linker".to_owned()])),
        ("lexer".to_owned(), BTreeSet::new()),
    ]);
    assert_eq!(
        adamantium_testing::affected_packages(&changed, &graph),
        BTreeSet::from(["compiler".to_owned(), "linker".to_owned()])
    );
}

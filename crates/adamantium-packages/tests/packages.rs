use adamantium_packages::{Lockfile, Manifest, Requirement, Version, resolve};
use std::collections::BTreeMap;

#[test]
fn formats_release_asset() {
    assert_eq!(
        adamantium_packages::release_asset("1.2.0").unwrap(),
        "adamantium_packet_1_2_0.wasm"
    );
}

#[test]
fn versions_are_exact_and_canonical() {
    assert_eq!("1.20.3".parse::<Version>().unwrap().to_string(), "1.20.3");
    for invalid in ["1", "1.2", "1.2.3.4", "01.2.3", "1.2.3-beta", "a.2.3"] {
        assert!(invalid.parse::<Version>().is_err(), "accepted {invalid}");
    }
}

fn manifest(name: &str, version: &str, dependencies: &[(&str, &str)]) -> Manifest {
    let dependencies = dependencies
        .iter()
        .map(|(source, version)| format!("\"{source}\" = \"{version}\""))
        .collect::<Vec<_>>()
        .join("\n");
    Manifest::parse(&format!(
        "[package]\nname = \"{name}\"\nversion = \"{version}\"\nabi = \"wasi-command-v1\"\n\n[dependencies]\n{dependencies}"
    ))
    .unwrap()
}

#[test]
fn validates_complete_metadata() {
    let value = Manifest::parse(
        r#"[package]
name = "Files"
version = "1.2.3"
abi = "wasi-command-v1"
description = "File helpers"
authors = ["Adam"]
license = "MIT"
repository = "https://github.com/example/Files"

[dependencies]
"https://github.com/example/Core" = "2.0.0"
"#,
    )
    .unwrap();
    assert_eq!(value.dependencies.len(), 1);
    assert!(
        Manifest::parse("[package]\nname='bad name'\nversion='1.0.0'\nabi='wasi-command-v1'")
            .is_err()
    );
    assert!(Manifest::parse("[package]\nname='Good'\nversion='1.0.0'\nabi='unknown'").is_err());
}

#[test]
fn resolves_transitive_dependencies_and_round_trips_lockfile() {
    let root = Requirement::new("https://github.com/example/App", "1.0.0").unwrap();
    let mut manifests = BTreeMap::new();
    manifests.insert(
        root.source.clone(),
        manifest(
            "App",
            "1.0.0",
            &[("https://github.com/example/Core", "2.0.0")],
        ),
    );
    manifests.insert(
        "https://github.com/example/Core".into(),
        manifest("Core", "2.0.0", &[]),
    );
    let lock = resolve(&[root], &manifests).unwrap();
    assert_eq!(
        lock.packages
            .iter()
            .map(|item| item.name.as_str())
            .collect::<Vec<_>>(),
        ["Core", "App"]
    );
    assert_eq!(Lockfile::parse(&lock.render().unwrap()).unwrap(), lock);
}

#[test]
fn detects_dependency_cycles() {
    let root = Requirement::new("https://github.com/example/A", "1.0.0").unwrap();
    let mut manifests = BTreeMap::new();
    manifests.insert(
        root.source.clone(),
        manifest("A", "1.0.0", &[("https://github.com/example/B", "1.0.0")]),
    );
    manifests.insert(
        "https://github.com/example/B".into(),
        manifest("B", "1.0.0", &[("https://github.com/example/A", "1.0.0")]),
    );
    assert!(resolve(&[root], &manifests).unwrap_err().contains("cycle"));
}

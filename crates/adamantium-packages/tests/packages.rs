use adamantium_packages::{Lockfile, Manifest, Requirement, Version, resolve};
use std::collections::BTreeMap;
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

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
    assert_eq!("nightly".parse::<Version>().unwrap().to_string(), "nightly");
    for invalid in ["1", "1.2", "1.2.3.4", "01.2.3", "1.2.3-beta", "a.2.3"] {
        assert!(invalid.parse::<Version>().is_err(), "accepted {invalid}");
    }
}

#[test]
fn nightly_requirements_accept_concrete_release_manifests() {
    let root = Requirement::new("https://github.com/example/App", "nightly").unwrap();
    let mut manifests = BTreeMap::new();
    manifests.insert(root.source.clone(), manifest("App", "1.2.3", &[]));
    let lock = resolve(&[root], &manifests).unwrap();
    assert_eq!(lock.packages[0].version, "nightly");
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

#[test]
fn generates_reproducible_release_assets_and_metadata() {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = std::env::temp_dir().join(format!(
        "adamantium-package-release-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    let manifest = root.join("source.toml");
    let wasm = root.join("source.wasm");
    fs::write(
        &manifest,
        "[package]\nname='Example'\nversion='1.2.3'\nabi='wasi-command-v1'\n",
    )
    .unwrap();
    fs::write(
        &wasm,
        wat::parse_str("(module (func (export \"_start\")))").unwrap(),
    )
    .unwrap();
    let bundle =
        adamantium_packages::generate_release(&manifest, &wasm, &root.join("release")).unwrap();
    assert_eq!(bundle.tag, "adamantium_packet_1_2_3");
    assert_eq!(bundle.assets.len(), 4);
    let checksums = fs::read_to_string(root.join("release/SHA256SUMS")).unwrap();
    assert!(checksums.contains("adamantium_packet.wasm"));
    let metadata: adamantium_packages::ReleaseMetadata = serde_json::from_str(
        &fs::read_to_string(root.join("release/adamantium_packet.release.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(metadata.name, "Example");
    assert_eq!(metadata.format, 1);
    fs::remove_dir_all(root).unwrap();
}

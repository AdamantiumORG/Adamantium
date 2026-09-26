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

#[test]
fn verifies_release_checksums_strictly() {
    let bytes = b"package bytes";
    let hash = adamantium_packages::sha256(bytes);
    let checksums = format!("{hash}  adamantium_packet.wasm\n");
    adamantium_packages::verify_checksum(&checksums, "adamantium_packet.wasm", bytes).unwrap();

    assert!(
        adamantium_packages::verify_checksum(&checksums, "adamantium_packet.wasm", b"tampered")
            .unwrap_err()
            .contains("checksum mismatch")
    );
    assert!(
        adamantium_packages::verify_checksum(&checksums, "adamantium_packet.toml", bytes)
            .unwrap_err()
            .contains("missing checksum")
    );
    assert!(
        adamantium_packages::verify_checksum("bad line", "adamantium_packet.wasm", bytes).is_err()
    );
}

#[test]
fn rejects_unsafe_package_sources() {
    for source in [
        "http://github.com/example/App",
        "https://github.com/example/App/extra",
        "https://github.com/example/App?download=1",
        "https://github.com/example/../App",
        "https://github.com/-example/App",
        "https://github.com/example/App/",
        " https://github.com/example/App",
    ] {
        assert!(
            Requirement::new(source, "1.0.0").is_err(),
            "accepted {source}"
        );
    }
    let requirement = Requirement::new("https://github.com/example/App.git", "1.0.0").unwrap();
    assert_eq!(requirement.source, "https://github.com/example/App");
}

#[test]
fn rejects_malicious_manifest_metadata() {
    let oversized = "a".repeat(adamantium_packages::MAX_MANIFEST_BYTES + 1);
    assert!(Manifest::parse(&oversized).unwrap_err().contains("exceeds"));
    let control = "[package]\nname='Good'\nversion='1.0.0'\nabi='wasi-command-v1'\ndescription='bad\u{7}value'\n";
    assert!(Manifest::parse(control).is_err());
    let authors = (0..65)
        .map(|index| format!("'author{index}'"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "[package]\nname='Good'\nversion='1.0.0'\nabi='wasi-command-v1'\nauthors=[{authors}]\n"
    );
    assert!(Manifest::parse(&source).unwrap_err().contains("64 authors"));
}

#[test]
fn rejects_path_traversal_and_hostile_checksum_entries() {
    for path in [
        "../packet.wasm",
        "..\\packet.wasm",
        "/packet.wasm",
        "C:\\packet.wasm",
        "safe//packet.wasm",
    ] {
        assert!(
            adamantium_packages::validate_relative_package_path(path).is_err(),
            "accepted {path}"
        );
    }
    let bytes = b"package bytes";
    let hash = adamantium_packages::sha256(bytes);
    for checksums in [
        format!("{hash}  ../adamantium_packet.toml\n{hash}  adamantium_packet.wasm\n"),
        format!("{hash}  other.wasm\n{hash}  other.wasm\n{hash}  adamantium_packet.wasm\n"),
    ] {
        assert!(
            adamantium_packages::verify_checksum(&checksums, "adamantium_packet.wasm", bytes)
                .is_err()
        );
    }
}

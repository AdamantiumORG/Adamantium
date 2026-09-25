use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    nightly: bool,
}

impl FromStr for Version {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value == "nightly" {
            return Ok(Self {
                major: 0,
                minor: 0,
                patch: 0,
                nightly: true,
            });
        }
        let parts = value.split('.').collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err("version must have the form MAJOR.MINOR.PATCH".into());
        }
        let parse = |part: &str| {
            if part.is_empty() || (part.len() > 1 && part.starts_with('0')) {
                return Err("version components must be canonical integers".to_string());
            }
            part.parse::<u64>()
                .map_err(|_| "version components must be unsigned integers".to_string())
        };
        Ok(Self {
            major: parse(parts[0])?,
            minor: parse(parts[1])?,
            patch: parse(parts[2])?,
            nightly: false,
        })
    }
}

impl Version {
    pub fn is_nightly(&self) -> bool {
        self.nightly
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.nightly {
            formatter.write_str("nightly")
        } else {
            write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Requirement {
    pub source: String,
    pub name: String,
    pub version: Version,
}

impl Requirement {
    pub fn new(source: &str, version: &str) -> Result<Self, String> {
        let name = github_repository_name(source)?;
        Ok(Self {
            source: source.trim_end_matches('/').to_string(),
            name,
            version: version.parse()?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub abi: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub repository: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub package: Metadata,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
}

impl Manifest {
    pub fn parse(source: &str) -> Result<Self, String> {
        let manifest: Self =
            toml::from_str(source).map_err(|error| format!("invalid package manifest: {error}"))?;
        validate_identifier(&manifest.package.name, "package name")?;
        let version = manifest.package.version.parse::<Version>()?;
        if version.is_nightly() {
            return Err(
                "package manifests must declare a concrete MAJOR.MINOR.PATCH version".into(),
            );
        }
        if manifest.package.abi != "wasi-command-v1" {
            return Err(format!(
                "unsupported package ABI '{}'",
                manifest.package.abi
            ));
        }
        if manifest
            .package
            .authors
            .iter()
            .any(|author| author.trim().is_empty())
        {
            return Err("package authors cannot contain empty values".into());
        }
        if !manifest.package.repository.is_empty() {
            github_repository_name(&manifest.package.repository)?;
        }
        for (source, version) in &manifest.dependencies {
            Requirement::new(source, version)
                .map_err(|error| format!("invalid dependency '{source}': {error}"))?;
        }
        Ok(manifest)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LockedPackage {
    pub name: String,
    pub source: String,
    pub version: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Lockfile {
    #[serde(rename = "package", default)]
    pub packages: Vec<LockedPackage>,
}

impl Lockfile {
    pub fn parse(source: &str) -> Result<Self, String> {
        let lock: Self =
            toml::from_str(source).map_err(|error| format!("invalid adamantium.lock: {error}"))?;
        let mut seen = BTreeSet::new();
        for package in &lock.packages {
            validate_identifier(&package.name, "locked package name")?;
            Requirement::new(&package.source, &package.version)?;
            if !seen.insert(package.source.clone()) {
                return Err(format!("duplicate locked package '{}'", package.source));
            }
        }
        Ok(lock)
    }

    pub fn render(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|error| format!("cannot serialize lockfile: {error}"))
    }
}

pub fn resolve(
    roots: &[Requirement],
    manifests: &BTreeMap<String, Manifest>,
) -> Result<Lockfile, String> {
    fn visit(
        requirement: &Requirement,
        manifests: &BTreeMap<String, Manifest>,
        visiting: &mut Vec<String>,
        complete: &mut BTreeSet<String>,
        output: &mut Vec<LockedPackage>,
    ) -> Result<(), String> {
        if complete.contains(&requirement.source) {
            return Ok(());
        }
        if let Some(position) = visiting.iter().position(|item| item == &requirement.source) {
            let mut cycle = visiting[position..].to_vec();
            cycle.push(requirement.source.clone());
            return Err(format!("dependency cycle detected: {}", cycle.join(" -> ")));
        }
        let manifest = manifests
            .get(&requirement.source)
            .ok_or_else(|| format!("missing manifest for dependency '{}'", requirement.source))?;
        if !requirement.version.is_nightly()
            && manifest.package.version != requirement.version.to_string()
        {
            return Err(format!(
                "package '{}' requested version {}, but its manifest declares {}",
                requirement.name, requirement.version, manifest.package.version
            ));
        }
        visiting.push(requirement.source.clone());
        let mut dependencies = Vec::new();
        for (source, version) in &manifest.dependencies {
            let dependency = Requirement::new(source, version)?;
            dependencies.push(dependency.source.clone());
            visit(&dependency, manifests, visiting, complete, output)?;
        }
        visiting.pop();
        dependencies.sort();
        output.push(LockedPackage {
            name: manifest.package.name.clone(),
            source: requirement.source.clone(),
            version: requirement.version.to_string(),
            dependencies,
        });
        complete.insert(requirement.source.clone());
        Ok(())
    }

    let mut roots = roots.to_vec();
    roots.sort_by(|left, right| left.source.cmp(&right.source));
    let mut output = Vec::new();
    let mut complete = BTreeSet::new();
    for root in &roots {
        visit(root, manifests, &mut Vec::new(), &mut complete, &mut output)?;
    }
    Ok(Lockfile { packages: output })
}

pub fn github_repository_name(source: &str) -> Result<String, String> {
    let rest = source
        .strip_prefix("https://github.com/")
        .ok_or_else(|| {
            "package source must be an https://github.com/OWNER/REPOSITORY URL".to_string()
        })?
        .trim_end_matches('/');
    let parts = rest.split('/').collect::<Vec<_>>();
    if parts.len() != 2 || parts.iter().any(|part| part.is_empty()) {
        return Err("package source must identify one GitHub repository".into());
    }
    let repository = parts[1].trim_end_matches(".git");
    let valid_owner = parts[0]
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-');
    let valid_repository = !repository.is_empty()
        && repository.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
        });
    if !valid_owner || !valid_repository {
        return Err("package source contains an invalid GitHub owner or repository name".into());
    }
    Ok(repository.to_string())
}

fn validate_identifier(value: &str, label: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'));
    if valid {
        Ok(())
    } else {
        Err(format!("{label} '{value}' is invalid"))
    }
}

pub fn release_asset(version: &str) -> Result<String, adamantium_diagnostics::Diagnostic> {
    let _layout = adamantium_project::ProjectLayout::new(".");
    let version = version.parse::<Version>().map_err(|error| {
        adamantium_diagnostics::Diagnostic::at_stage(
            adamantium_diagnostics::Stage::Package,
            adamantium_diagnostics::Severity::Error,
            "E400",
            error,
            Default::default(),
        )
    })?;
    if version.is_nightly() {
        return Err(adamantium_diagnostics::Diagnostic::at_stage(
            adamantium_diagnostics::Stage::Package,
            adamantium_diagnostics::Severity::Error,
            "E610",
            "nightly is a release channel, not a package manifest version",
            Default::default(),
        ));
    }
    let _validates_wasm = adamantium_wasm::is_module(adamantium_wasm::MAGIC);
    Ok(format!(
        "adamantium_packet_{}_{}_{}.wasm",
        version.major, version.minor, version.patch
    ))
}

pub const PACKAGE_MANIFEST: &str = "adamantium_packet.toml";
pub const PACKAGE_WASM: &str = "adamantium_packet.wasm";
pub const PACKAGE_CHECKSUMS: &str = "SHA256SUMS";
pub const RELEASE_METADATA: &str = "adamantium_packet.release.json";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ReleaseMetadata {
    pub format: u32,
    pub name: String,
    pub version: String,
    pub abi: String,
    pub tag: String,
    pub wasm_sha256: String,
    pub manifest_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseBundle {
    pub directory: PathBuf,
    pub tag: String,
    pub assets: Vec<PathBuf>,
}

pub fn release_tag(version: &str) -> Result<String, String> {
    let version = version.parse::<Version>()?;
    if version.is_nightly() {
        return Err("nightly is a release channel, not a package manifest version".into());
    }
    Ok(format!(
        "adamantium_packet_{}_{}_{}",
        version.major, version.minor, version.patch
    ))
}

pub fn generate_release(
    manifest_path: &Path,
    wasm_path: &Path,
    output: &Path,
) -> Result<ReleaseBundle, String> {
    let manifest_source = fs::read_to_string(manifest_path)
        .map_err(|error| format!("could not read {}: {error}", manifest_path.display()))?;
    let manifest = Manifest::parse(&manifest_source)?;
    let wasm = fs::read(wasm_path)
        .map_err(|error| format!("could not read {}: {error}", wasm_path.display()))?;
    adamantium_wasm::validate_package(&wasm)
        .map_err(|error| format!("invalid package WASM: {error}"))?;
    let canonical_manifest = toml::to_string_pretty(&manifest)
        .map_err(|error| format!("cannot serialize package manifest: {error}"))?;
    let wasm_hash = sha256(&wasm);
    let manifest_hash = sha256(canonical_manifest.as_bytes());
    let tag = release_tag(&manifest.package.version)?;
    let metadata = ReleaseMetadata {
        format: 1,
        name: manifest.package.name.clone(),
        version: manifest.package.version.clone(),
        abi: manifest.package.abi.clone(),
        tag: tag.clone(),
        wasm_sha256: wasm_hash.clone(),
        manifest_sha256: manifest_hash.clone(),
    };
    fs::create_dir_all(output)
        .map_err(|error| format!("could not create {}: {error}", output.display()))?;
    let wasm_output = output.join(PACKAGE_WASM);
    let manifest_output = output.join(PACKAGE_MANIFEST);
    let checksums_output = output.join(PACKAGE_CHECKSUMS);
    let metadata_output = output.join(RELEASE_METADATA);
    fs::write(&wasm_output, wasm).map_err(|error| error.to_string())?;
    fs::write(&manifest_output, canonical_manifest).map_err(|error| error.to_string())?;
    fs::write(
        &checksums_output,
        format!("{wasm_hash}  {PACKAGE_WASM}\n{manifest_hash}  {PACKAGE_MANIFEST}\n"),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &metadata_output,
        serde_json::to_string_pretty(&metadata)
            .map_err(|error| format!("cannot serialize release metadata: {error}"))?,
    )
    .map_err(|error| error.to_string())?;
    Ok(ReleaseBundle {
        directory: output.to_path_buf(),
        tag,
        assets: vec![
            wasm_output,
            manifest_output,
            checksums_output,
            metadata_output,
        ],
    })
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

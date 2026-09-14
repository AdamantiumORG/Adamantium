use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
}

impl FromStr for Version {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
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
        })
    }
}

impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}.{}", self.major, self.minor, self.patch)
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
        manifest.package.version.parse::<Version>()?;
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
        if manifest.package.version != requirement.version.to_string() {
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
        adamantium_diagnostics::Diagnostic::error("E400", error, Default::default())
    })?;
    let _validates_wasm = adamantium_wasm::is_module(adamantium_wasm::MAGIC);
    Ok(format!(
        "adamantium_packet_{}_{}_{}.wasm",
        version.major, version.minor, version.patch
    ))
}

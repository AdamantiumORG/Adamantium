use serde::Deserialize;
use std::{
    collections::{BTreeSet, HashMap},
    env,
    path::Path,
    process::Command,
};

#[derive(Deserialize)]
struct Metadata {
    packages: Vec<Package>,
    resolve: Resolve,
    workspace_members: Vec<String>,
}
#[derive(Deserialize)]
struct Package {
    id: String,
    name: String,
    manifest_path: String,
}
#[derive(Deserialize)]
struct Resolve {
    nodes: Vec<Node>,
}
#[derive(Deserialize)]
struct Node {
    id: String,
    dependencies: Vec<String>,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let changed_files = if args.first().is_some_and(|arg| arg == "--changed") {
        args.into_iter().skip(1).collect()
    } else {
        git_changed_files(
            args.first().map(String::as_str),
            args.get(1).map(String::as_str),
        )
    };
    let output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()
        .expect("could not run cargo metadata");
    assert!(output.status.success(), "cargo metadata failed");
    let metadata: Metadata =
        serde_json::from_slice(&output.stdout).expect("invalid cargo metadata");
    let workspace_ids: BTreeSet<_> = metadata.workspace_members.iter().cloned().collect();
    let workspace_packages: Vec<_> = metadata
        .packages
        .iter()
        .filter(|package| workspace_ids.contains(&package.id))
        .collect();
    let all: BTreeSet<_> = workspace_packages
        .iter()
        .map(|package| package.name.clone())
        .collect();
    let shared_change = changed_files
        .iter()
        .any(|path| !path.replace('\\', "/").starts_with("crates/"));
    let direct = if shared_change || changed_files.is_empty() {
        all
    } else {
        workspace_packages
            .iter()
            .filter(|package| {
                let directory = Path::new(&package.manifest_path)
                    .parent()
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                changed_files.iter().any(|changed| {
                    directory.ends_with(
                        &changed
                            .replace('\\', "/")
                            .split('/')
                            .take(2)
                            .collect::<Vec<_>>()
                            .join("/"),
                    )
                })
            })
            .map(|package| package.name.clone())
            .collect()
    };
    let names: HashMap<_, _> = workspace_packages
        .iter()
        .map(|package| (package.id.clone(), package.name.clone()))
        .collect();
    let mut dependencies: HashMap<String, BTreeSet<String>> = metadata
        .resolve
        .nodes
        .into_iter()
        .filter_map(|node| {
            names.get(&node.id).map(|name| {
                (
                    name.clone(),
                    node.dependencies
                        .into_iter()
                        .filter_map(|id| names.get(&id).cloned())
                        .collect(),
                )
            })
        })
        .collect();
    dependencies
        .entry("adamantium-cli".to_owned())
        .or_default()
        .insert("adamantium-runtime".to_owned());
    let affected = adamantium_testing::affected_packages(&direct, &dependencies);
    println!("{}", serde_json::to_string(&affected).unwrap());
}

fn git_changed_files(base: Option<&str>, head: Option<&str>) -> Vec<String> {
    let range = match (base, head) {
        (Some(base), Some(head)) => format!("{base}..{head}"),
        _ => "HEAD~1..HEAD".to_owned(),
    };
    let output = Command::new("git")
        .args(["diff", "--name-only", &range])
        .output()
        .expect("could not run git diff");
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_owned)
        .collect()
}

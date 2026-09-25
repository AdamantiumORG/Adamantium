use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub professional: bool,
}

#[derive(Debug)]
pub struct ProjectLayout {
    pub root: PathBuf,
}

impl ProjectLayout {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    pub fn source(&self) -> PathBuf {
        self.root.join("code/main.ad")
    }
    pub fn validate(&self) -> Result<(), adamantium_diagnostics::Diagnostic> {
        if self.source().is_file() {
            Ok(())
        } else {
            Err(adamantium_diagnostics::Diagnostic::at_stage(
                adamantium_diagnostics::Stage::Project,
                adamantium_diagnostics::Severity::Error,
                "E300",
                "missing code/main.ad",
                Default::default(),
            ))
        }
    }
}

pub fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(
            "project name must contain only ASCII letters, digits, underscores or hyphens".into(),
        );
    }
    let upper = name.to_ascii_uppercase();
    if [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ]
    .contains(&upper.as_str())
    {
        return Err("project name is a reserved Windows device name".into());
    }
    Ok(())
}

pub fn create(root: &Path) -> Result<(), String> {
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("project path must end with a valid UTF-8 project name")?;
    validate_name(name)?;
    if root.exists() {
        return Err(format!("{} already exists", root.display()));
    }
    fs::create_dir_all(root.join("code"))
        .map_err(|e| format!("could not create {}: {e}", root.display()))?;
    fs::create_dir(root.join("target"))
        .map_err(|e| format!("could not create target directory: {e}"))?;
    fs::write(
        root.join("project.toml"),
        format!("name = \"{name}\"\nversion = \"0.1.0\"\ndescription = \"\"\nauthors = []\nprofessional = false\n"),
    )
    .map_err(|e| format!("could not create project.toml: {e}"))?;
    fs::write(root.join("requirement.toml"), "[packages]\n")
        .map_err(|e| format!("could not create requirement.toml: {e}"))?;
    fs::write(
        root.join("code/main.ad"),
        "fun main() {\n    print.newline(\"Hello, Adamantium!\");\n}\n",
    )
    .map_err(|e| format!("could not create code/main.ad: {e}"))?;
    fs::write(root.join(".gitignore"), "/target/\n/packages/\n")
        .map_err(|e| format!("could not create .gitignore: {e}"))?;
    Ok(())
}

pub fn initialize(root: &Path) -> Result<(), String> {
    if !root.is_dir() {
        return Err(format!("{} is not an existing directory", root.display()));
    }
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("project path must end with a valid UTF-8 project name")?;
    validate_name(name)?;
    for manifest in ["project.toml", "requirement.toml"] {
        if root.join(manifest).exists() {
            return Err(format!(
                "{} already contains {manifest}; refusing to overwrite an existing project",
                root.display()
            ));
        }
    }
    let code = root.join("code");
    fs::create_dir_all(&code).map_err(|e| format!("could not create {}: {e}", code.display()))?;
    let main = code.join("main.ad");
    if !main.exists() {
        fs::write(
            &main,
            "fun main() {\n    print.newline(\"Hello, Adamantium!\");\n}\n",
        )
        .map_err(|e| format!("could not create {}: {e}", main.display()))?;
    }
    fs::write(
        root.join("project.toml"),
        format!("name = \"{name}\"\nversion = \"0.1.0\"\ndescription = \"\"\nauthors = []\nprofessional = false\n"),
    )
    .map_err(|e| format!("could not create project.toml: {e}"))?;
    fs::write(root.join("requirement.toml"), "[packages]\n")
        .map_err(|e| format!("could not create requirement.toml: {e}"))?;
    let gitignore = root.join(".gitignore");
    if !gitignore.exists() {
        fs::write(gitignore, "/target/\n/packages/\n")
            .map_err(|e| format!("could not create .gitignore: {e}"))?;
    }
    Ok(())
}

pub fn read_toml(path: &Path) -> Result<toml::Table, String> {
    fs::read_to_string(path)
        .map_err(|e| format!("{}: {e}", path.display()))?
        .parse()
        .map_err(|e| format!("{}: {e}", path.display()))
}

pub fn read_manifest(root: &Path) -> Result<Manifest, Vec<String>> {
    let table = read_toml(&root.join("project.toml")).map_err(|e| vec![e])?;
    let mut errors = Vec::new();
    let name = string_field(&table, "name", &mut errors);
    if let Some(name) = &name
        && let Err(error) = validate_name(name)
    {
        errors.push(format!("project.toml: {error}"));
    }
    let version = string_field(&table, "version", &mut errors);
    let description = string_field(&table, "description", &mut errors);
    let authors = table
        .get("authors")
        .and_then(toml::Value::as_array)
        .filter(|a| a.iter().all(toml::Value::is_str))
        .map(|a| {
            a.iter()
                .filter_map(toml::Value::as_str)
                .map(str::to_owned)
                .collect()
        });
    if authors.is_none() {
        errors.push("project.toml: authors must be an array of strings".into());
    }
    let professional = match table.get("professional") {
        Some(value) => match value.as_bool() {
            Some(value) => value,
            None => {
                errors.push("project.toml: professional must be a boolean".into());
                false
            }
        },
        None => false,
    };
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(Manifest {
        name: name.unwrap(),
        version: version.unwrap(),
        description: description.unwrap(),
        authors: authors.unwrap(),
        professional,
    })
}

fn string_field(table: &toml::Table, field: &str, errors: &mut Vec<String>) -> Option<String> {
    let value = table
        .get(field)
        .and_then(toml::Value::as_str)
        .map(str::to_owned);
    if value.is_none() {
        errors.push(format!("project.toml: {field} must be a string"));
    }
    value
}

pub fn load_modules<F>(code: &Path, dependencies: F) -> Result<Vec<(String, String)>, String>
where
    F: Fn(&str) -> Result<Vec<String>, String>,
{
    fn visit<F>(
        module: &str,
        code: &Path,
        dependencies: &F,
        visiting: &mut Vec<String>,
        loaded: &mut HashSet<String>,
        result: &mut Vec<(String, String)>,
    ) -> Result<(), String>
    where
        F: Fn(&str) -> Result<Vec<String>, String>,
    {
        if loaded.contains(module) {
            return Ok(());
        }
        if let Some(start) = visiting.iter().position(|item| item == module) {
            let mut cycle = visiting[start..].to_vec();
            cycle.push(module.to_owned());
            return Err(format!("circular pack dependency: {}", cycle.join(" -> ")));
        }
        if !valid_module_path(module) {
            return Err(format!("invalid module path '{module}'"));
        }
        let path = if module.is_empty() {
            code.join("main.ad")
        } else {
            code.join(format!("{module}.ad"))
        };
        let source = fs::read_to_string(&path).map_err(|e| {
            format!(
                "could not load module '{}': {}: {e}",
                if module.is_empty() { "main" } else { module },
                path.display()
            )
        })?;
        visiting.push(module.to_owned());
        for dependency in dependencies(&source).map_err(|e| format!("{}:{e}", path.display()))? {
            visit(&dependency, code, dependencies, visiting, loaded, result)?;
        }
        visiting.pop();
        loaded.insert(module.to_owned());
        result.push((module.to_owned(), source));
        Ok(())
    }
    let mut result = Vec::new();
    visit(
        "",
        code,
        &dependencies,
        &mut Vec::new(),
        &mut HashSet::new(),
        &mut result,
    )?;
    Ok(result)
}

fn valid_module_path(module: &str) -> bool {
    module.is_empty()
        || module.split('/').all(|part| {
            let mut chars = part.chars();
            chars
                .next()
                .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
                && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    fn temp() -> PathBuf {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "adamantium-project-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }
    #[test]
    fn creates_and_reads_project() {
        let root = temp().join("Example");
        create(&root).unwrap();
        assert_eq!(read_manifest(&root).unwrap().name, "Example");
        fs::remove_dir_all(root.parent().unwrap()).unwrap();
    }
    #[test]
    fn loads_dependencies_first() {
        let root = temp();
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("main.ad"), "pack utils;").unwrap();
        fs::write(root.join("utils.ad"), "fun helper() {}").unwrap();
        let modules = load_modules(&root, |s| {
            Ok(if s.contains("pack utils") {
                vec!["utils".into()]
            } else {
                vec![]
            })
        })
        .unwrap();
        assert_eq!(
            modules.iter().map(|m| m.0.as_str()).collect::<Vec<_>>(),
            ["utils", ""]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn initializes_an_existing_directory_without_overwriting_source() {
        let root = temp().join("Existing");
        fs::create_dir_all(root.join("code")).unwrap();
        fs::write(root.join("code/main.ad"), "fun main() {}\n").unwrap();
        initialize(&root).unwrap();
        assert_eq!(
            fs::read_to_string(root.join("code/main.ad")).unwrap(),
            "fun main() {}\n"
        );
        assert!(root.join("project.toml").is_file());
        assert!(
            initialize(&root)
                .unwrap_err()
                .contains("refusing to overwrite")
        );
        fs::remove_dir_all(root.parent().unwrap()).unwrap();
    }
}

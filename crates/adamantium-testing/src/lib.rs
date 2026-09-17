use std::{
    collections::{BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_PROJECT: AtomicUsize = AtomicUsize::new(0);

pub trait LanguageCompiler {
    fn build(&self, project: &Path) -> Result<PathBuf, String>;
    fn analyze(&self, project: &Path) -> Result<(), String>;
    fn render_diagnostics(&self, diagnostics: &str) -> String;
}

struct TemporaryProject(PathBuf);

impl Drop for TemporaryProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct Case {
    name: String,
    directory: PathBuf,
    valid: bool,
}

pub fn run_language_tests(
    root: &Path,
    verbose: bool,
    compiler: &impl LanguageCompiler,
) -> Result<ExitCode, String> {
    let root = root
        .canonicalize()
        .map_err(|error| format!("{}: {error}", root.display()))?;
    let mut cases = Vec::new();
    discover(&root, &root, &mut cases)?;
    if cases.is_empty() {
        return Err(format!("{}: no language test cases found", root.display()));
    }
    cases.sort_by(|left, right| left.name.cmp(&right.name));
    println!("running {} language tests", cases.len());
    let mut passed = 0;
    for case in &cases {
        match run_case(case, compiler) {
            Ok(()) => {
                passed += 1;
                println!("test {} ... ok", case.name);
            }
            Err(error) => {
                println!("test {} ... FAILED", case.name);
                if verbose || !error.is_empty() {
                    eprintln!("{error}");
                }
            }
        }
    }
    let failed = cases.len() - passed;
    println!(
        "\nlanguage test result: {}. {passed} passed; {failed} failed",
        if failed == 0 { "ok" } else { "FAILED" }
    );
    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn discover(root: &Path, directory: &Path, cases: &mut Vec<Case>) -> Result<(), String> {
    if directory.join("program.ad").is_file() {
        let relative = directory.strip_prefix(root).unwrap();
        let valid = relative
            .components()
            .next()
            .is_some_and(|part| part.as_os_str() == "valid");
        let invalid = relative
            .components()
            .next()
            .is_some_and(|part| part.as_os_str() == "invalid");
        if !valid && !invalid {
            return Err(format!(
                "{}: language test must be under valid or invalid",
                directory.display()
            ));
        }
        for required in ["expected.txt", "expected_exit_code"] {
            if !directory.join(required).is_file() {
                return Err(format!("{}: missing {required}", directory.display()));
            }
        }
        cases.push(Case {
            name: relative.to_string_lossy().replace('\\', "/"),
            directory: directory.to_owned(),
            valid,
        });
        return Ok(());
    }
    let mut entries = fs::read_dir(directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{}: {error}", directory.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        if entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            discover(root, &entry.path(), cases)?;
        }
    }
    Ok(())
}

fn run_case(case: &Case, compiler: &impl LanguageCompiler) -> Result<(), String> {
    let expected_code = fs::read_to_string(case.directory.join("expected_exit_code"))
        .map_err(|error| error.to_string())?
        .trim()
        .parse::<i32>()
        .map_err(|error| format!("{}: {error}", case.name))?;
    let expected = normalize(
        &fs::read_to_string(case.directory.join("expected.txt"))
            .map_err(|error| error.to_string())?,
    );
    let project = create_project(case)?;
    let (code, output) = if case.valid {
        match compiler.build(&project.0) {
            Ok(executable) => {
                let output = Command::new(executable)
                    .current_dir(&project.0)
                    .output()
                    .map_err(|error| format!("could not run {}: {error}", case.name))?;
                (
                    output.status.code().unwrap_or(1),
                    if output.status.success() {
                        String::from_utf8_lossy(&output.stdout).into_owned()
                    } else {
                        String::from_utf8_lossy(&output.stderr).into_owned()
                    },
                )
            }
            Err(error) => (1, compiler.render_diagnostics(&error)),
        }
    } else {
        match compiler.analyze(&project.0) {
            Ok(()) => (0, String::new()),
            Err(error) => (1, compiler.render_diagnostics(&error)),
        }
    };
    let output = normalize(&output.replace(&project.0.to_string_lossy().to_string(), "<project>"))
        .replace("code/main.ad", "program.ad");
    if code != expected_code {
        return Err(format!(
            "expected exit code {expected_code}, found {code}\noutput:\n{output}"
        ));
    }
    if case.valid {
        if output != expected {
            return Err(format!("expected:\n{expected}\nfound:\n{output}"));
        }
    } else {
        for expected_line in expected.lines().filter(|line| !line.trim().is_empty()) {
            if !output.contains(expected_line) {
                return Err(format!(
                    "missing expected diagnostic fragment '{expected_line}'\noutput:\n{output}"
                ));
            }
        }
    }
    Ok(())
}

fn create_project(case: &Case) -> Result<TemporaryProject, String> {
    let root = std::env::temp_dir().join(format!(
        "adamantium-language-test-{}-{}",
        std::process::id(),
        NEXT_PROJECT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("code")).map_err(|error| error.to_string())?;
    fs::write(
        root.join("project.toml"),
        "name=\"LanguageTest\"\nversion=\"1.0.0\"\ndescription=\"\"\nauthors=[]\n",
    )
    .map_err(|error| error.to_string())?;
    fs::write(root.join("requirement.toml"), "[packages]\n").map_err(|error| error.to_string())?;
    for entry in fs::read_dir(&case.directory)
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?
    {
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "ad") {
            let destination = if path.file_name().is_some_and(|name| name == "program.ad") {
                root.join("code/main.ad")
            } else {
                root.join("code").join(path.file_name().unwrap())
            };
            fs::copy(path, destination).map_err(|error| error.to_string())?;
        }
    }
    Ok(TemporaryProject(root))
}

fn normalize(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\\', "/")
}

pub fn affected_packages(
    changed: &BTreeSet<String>,
    dependencies: &HashMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut affected = changed.clone();
    loop {
        let downstream: Vec<_> = dependencies
            .iter()
            .filter(|(_, direct)| {
                direct
                    .iter()
                    .any(|dependency| affected.contains(dependency))
            })
            .map(|(package, _)| package.clone())
            .collect();
        let old_len = affected.len();
        affected.extend(downstream);
        if affected.len() == old_len {
            return affected;
        }
    }
}

pub fn architecture_is_connected() -> bool {
    !adamantium_compiler::pipeline_layers().is_empty()
        && adamantium_project::ProjectLayout::new(".")
            .source()
            .ends_with("code/main.ad")
        && adamantium_diagnostics::Severity::Error != adamantium_diagnostics::Severity::Warning
}

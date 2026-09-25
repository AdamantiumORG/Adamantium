mod codegen;
mod diagnostics;
mod formatter;
mod optimizer;
mod packages;
mod syntax;
mod typed;
#[allow(dead_code)]
mod types {
    pub use adamantium_types::*;
}

use std::{
    collections::VecDeque,
    env,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

fn main() -> ExitCode {
    debug_assert!(!adamantium_compiler::pipeline_layers().is_empty());
    let arguments = env::args_os().skip(1).collect();
    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| cli(arguments)));
    std::panic::set_hook(panic_hook);
    match result.unwrap_or_else(|_| {
        Err("internal compiler error: the frontend rejected input unexpectedly; please report this source file".into())
    }) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{}", diagnostics::render_errors(&error));
            ExitCode::FAILURE
        }
    }
}

struct CliLanguageCompiler;

impl adamantium_testing::LanguageCompiler for CliLanguageCompiler {
    fn build(&self, project: &Path) -> Result<PathBuf, String> {
        build(project, optimizer::Level::default())
    }

    fn analyze(&self, project: &Path) -> Result<(), String> {
        analyze(project).map(|_| ())
    }

    fn render_diagnostics(&self, errors: &str) -> String {
        diagnostics::render_errors(errors)
    }
}

const HELP: &str = "Adamantium compiler (Windows/Linux x86-64)\n\
Usage:\n\
  adamantium check [PROJECT_DIRECTORY]\n\
  adamantium doctor [PROJECT_DIRECTORY]\n\
  adamantium fmt [PROJECT_DIRECTORY]\n\
  adamantium install [PROJECT_DIRECTORY]\n\
  adamantium package prepare [PACKAGE_DIRECTORY]\n\
  adamantium package publish [PACKAGE_DIRECTORY]\n\
  adamantium clean [PROJECT_DIRECTORY]\n\
  adamantium clear [PROJECT_DIRECTORY]\n\
  adamantium build [PROJECT_DIRECTORY] [-O0|-O1|-O2]\n\
  adamantium run [PROJECT_DIRECTORY] [-O0|-O1|-O2] [--name value ...]\n\
  adamantium test list [PROJECT_DIRECTORY]\n\
  adamantium test run [PROJECT_DIRECTORY] [TEST_NAME] [--verbose]\n\
  adamantium test language [SUITE_DIRECTORY] [--verbose]\n\
  adamantium new <PROJECT_NAME_OR_PATH>\n\
  adamantium --help\n\
  adamantium --version\n\n\
PROJECT_DIRECTORY defaults to the current directory.\n\
For compatibility, `adamantium PROJECT_DIRECTORY` is the same as `adamantium build PROJECT_DIRECTORY`.\n\
The portable Windows and Linux packages include NASM and a linker. Source builds require NASM and a platform linker.\n\
Override tools with ADAMANTIUM_NASM and ADAMANTIUM_LINKER.";

enum Action {
    Help,
    Version,
    Check(PathBuf),
    Doctor(PathBuf),
    Format(PathBuf),
    Install(PathBuf),
    PackagePrepare(PathBuf),
    PackagePublish(PathBuf),
    Clean(PathBuf),
    TestList(PathBuf),
    TestRun(PathBuf, Option<String>, bool),
    LanguageTests(PathBuf, bool),
    Build(PathBuf, optimizer::Level),
    Run(PathBuf, optimizer::Level, Vec<OsString>),
    New(PathBuf),
}

type SourceFiles = Vec<(String, String)>;
type ProjectSources = (PathBuf, String, SourceFiles, Vec<packages::Binding>, bool);

#[derive(Debug, PartialEq)]
struct Package {
    name: String,
    source: String,
    version: String,
}

fn cli(args: Vec<OsString>) -> Result<ExitCode, String> {
    match action(args)? {
        Action::Help => println!("{HELP}"),
        Action::Version => println!("adamantium {}", env!("CARGO_PKG_VERSION")),
        Action::Check(root) => {
            check(&root)?;
            println!("Checked {}", root.display());
        }
        Action::Doctor(root) => return doctor(&root),
        Action::Format(root) => {
            let changed = format_project(&root)?;
            println!("Formatted {changed} source file(s) in {}", root.display());
        }
        Action::Install(root) => install_packages(&root)?,
        Action::PackagePrepare(root) => {
            let bundle = prepare_package_release(&root)?;
            println!("Prepared {} at {}", bundle.tag, bundle.directory.display());
        }
        Action::PackagePublish(root) => publish_package_release(&root)?,
        Action::Clean(root) => clean_project(&root)?,
        Action::TestList(root) => list_tests(&root)?,
        Action::TestRun(root, filter, verbose) => {
            return run_tests(&root, filter.as_deref(), verbose);
        }
        Action::LanguageTests(root, verbose) => {
            return adamantium_testing::run_language_tests(&root, verbose, &CliLanguageCompiler);
        }
        Action::Build(root, level) => {
            let executable = build(&root, level)?;
            println!("Built {}", executable.display());
        }
        Action::Run(root, level, arguments) => {
            let executable = build(&root, level)?;
            eprintln!("Built {}", executable.display());
            let status = Command::new(&executable)
                .current_dir(&root)
                .args(arguments)
                .status()
                .map_err(|e| format!("could not run {}: {e}", executable.display()))?;
            return Ok(status
                .code()
                .and_then(|code| u8::try_from(code).ok())
                .map_or(ExitCode::FAILURE, ExitCode::from));
        }
        Action::New(root) => {
            create_project(&root)?;
            println!("Created Adamantium project at {}", root.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn action(args: Vec<OsString>) -> Result<Action, String> {
    let mut args = args.into_iter();
    let Some(first) = args.next() else {
        return Ok(Action::Help);
    };
    if first == "--help" || first == "-h" {
        no_more_args(args)?;
        return Ok(Action::Help);
    }
    if first == "--version" || first == "-V" {
        no_more_args(args)?;
        return Ok(Action::Version);
    }
    if first == "new" {
        let root = args
            .next()
            .ok_or("adamantium new requires a project name or path; use --help")?;
        no_more_args(args)?;
        return Ok(Action::New(root.into()));
    }
    if first == "build" {
        let (level, remaining) = optimization_arguments(args.collect())?;
        let root = match remaining.as_slice() {
            [] => current_directory()?.into(),
            [root] => root.into(),
            _ => return Err("too many arguments for 'adamantium build'; use --help".into()),
        };
        return Ok(Action::Build(root, level));
    }
    if first == "check" {
        let root = args.next().map_or_else(current_directory, Ok)?;
        no_more_args(args)?;
        return Ok(Action::Check(root.into()));
    }
    if first == "doctor" {
        let root = args.next().map_or_else(current_directory, Ok)?;
        no_more_args(args)?;
        return Ok(Action::Doctor(root.into()));
    }
    if first == "fmt" {
        let root = args.next().map_or_else(current_directory, Ok)?;
        no_more_args(args)?;
        return Ok(Action::Format(root.into()));
    }
    if first == "install" {
        let root = args.next().map_or_else(current_directory, Ok)?;
        no_more_args(args)?;
        return Ok(Action::Install(root.into()));
    }
    if first == "package" {
        let command = args
            .next()
            .ok_or("adamantium package requires 'prepare' or 'publish'; use --help")?;
        let root = args.next().map_or_else(current_directory, Ok)?;
        no_more_args(args)?;
        return match command.to_string_lossy().as_ref() {
            "prepare" => Ok(Action::PackagePrepare(root.into())),
            "publish" => Ok(Action::PackagePublish(root.into())),
            _ => Err("adamantium package requires 'prepare' or 'publish'; use --help".into()),
        };
    }
    if first == "clean" || first == "clear" {
        let root = args.next().map_or_else(current_directory, Ok)?;
        no_more_args(args)?;
        return Ok(Action::Clean(root.into()));
    }
    if first == "run" {
        let (level, remaining) = optimization_arguments(args.collect())?;
        let (root, arguments) = if remaining
            .first()
            .is_some_and(|argument| !argument.to_string_lossy().starts_with('-'))
        {
            (PathBuf::from(&remaining[0]), remaining[1..].to_vec())
        } else {
            (PathBuf::from(current_directory()?), remaining)
        };
        return Ok(Action::Run(root, level, arguments));
    }
    if first == "test" {
        let command = args
            .next()
            .ok_or("adamantium test requires 'list', 'run', or 'language'; use --help")?;
        let remaining = args.collect::<Vec<_>>();
        if command == "language" {
            let verbose = args_contains_verbose(&remaining);
            let paths = remaining
                .into_iter()
                .filter(|value| value != "--verbose")
                .collect::<Vec<_>>();
            let root = match paths.as_slice() {
                [] => PathBuf::from(current_directory()?).join("tests"),
                [root] => root.into(),
                _ => {
                    return Err(
                        "too many arguments for 'adamantium test language'; use --help".into(),
                    );
                }
            };
            return Ok(Action::LanguageTests(root, verbose));
        }
        if command == "list" {
            let root = remaining
                .first()
                .map_or_else(current_directory, |value| Ok(value.clone()))?;
            if remaining.len() > 1 {
                return Err("too many arguments for 'adamantium test list'; use --help".into());
            }
            return Ok(Action::TestList(root.into()));
        }
        if command == "run" {
            let verbose = args_contains_verbose(&remaining);
            let mut values = remaining.into_iter().filter(|value| value != "--verbose");
            let values = values.by_ref().collect::<Vec<_>>();
            let (root, filter) = match values.as_slice() {
                [] => (PathBuf::from(current_directory()?), None),
                [one] if Path::new(one).join("project.toml").is_file() => (one.into(), None),
                [one] => (
                    PathBuf::from(current_directory()?),
                    Some(one.to_string_lossy().into_owned()),
                ),
                [root, test] => (root.into(), Some(test.to_string_lossy().into_owned())),
                _ => return Err("too many arguments for 'adamantium test run'; use --help".into()),
            };
            return Ok(Action::TestRun(root, filter, verbose));
        }
        let command = command.to_string_lossy();
        let help = closest_name(&command, &["language", "list", "run"])
            .map(|name| format!(" Did you mean '{name}'?"))
            .unwrap_or_default();
        return Err(format!(
            "unknown test command '{command}'.{help} use --help."
        ));
    }
    if first.to_string_lossy().starts_with('-') {
        return Err(format!(
            "unknown option '{}'; use --help",
            first.to_string_lossy()
        ));
    }
    let text = first.to_string_lossy();
    if !Path::new(&first).exists()
        && let Some(command) = closest_name(
            &text,
            &[
                "build", "check", "clean", "clear", "doctor", "fmt", "install", "new", "run",
                "test",
            ],
        )
    {
        return Err(format!(
            "unknown command '{text}'. Did you mean '{command}'? use --help."
        ));
    }
    no_more_args(args)?;
    Ok(Action::Build(first.into(), optimizer::Level::default()))
}

fn optimization_arguments(
    args: Vec<OsString>,
) -> Result<(optimizer::Level, Vec<OsString>), String> {
    let mut level = None;
    let mut remaining = Vec::new();
    for argument in args {
        let text = argument.to_string_lossy();
        if let Some(parsed) = optimizer::Level::parse(&text) {
            if level.replace(parsed).is_some() {
                return Err("only one optimization level may be specified".into());
            }
        } else if text.starts_with("-O") {
            return Err(format!(
                "unknown optimization level '{text}'; expected -O0, -O1, or -O2"
            ));
        } else {
            remaining.push(argument);
        }
    }
    Ok((level.unwrap_or_default(), remaining))
}

fn closest_name<'a>(input: &str, choices: &'a [&str]) -> Option<&'a str> {
    choices
        .iter()
        .map(|choice| (*choice, edit_distance(input, choice)))
        .filter(|(_, distance)| *distance <= 2)
        .min_by_key(|(_, distance)| *distance)
        .map(|(choice, _)| choice)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut row = (0..=right.chars().count()).collect::<Vec<_>>();
    for (left_index, left_char) in left.chars().enumerate() {
        let mut previous = row[0];
        row[0] = left_index + 1;
        for (right_index, right_char) in right.chars().enumerate() {
            let replaced = previous + usize::from(left_char != right_char);
            previous = row[right_index + 1];
            row[right_index + 1] = (row[right_index + 1] + 1)
                .min(row[right_index] + 1)
                .min(replaced);
        }
    }
    row[right.chars().count()]
}

fn args_contains_verbose(args: &[OsString]) -> bool {
    args.iter().any(|value| value == "--verbose")
}

#[derive(Clone)]
struct TestDefinition {
    name: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum StopOnFailed {
    #[default]
    Disabled,
    StopStarted,
    DontStopStarted,
}

#[derive(Default)]
struct TestOptions {
    parallel: Option<usize>,
    stop_on_failed: StopOnFailed,
}

fn test_source(root: &Path) -> Result<(String, Vec<TestDefinition>, TestOptions), String> {
    let path = root.join("code/tests.ad");
    let source =
        fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut output = String::new();
    let mut tests = Vec::new();
    let mut options = TestOptions::default();
    let mut awaiting_test = false;
    let mut found_declaration = false;
    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        let mut output_line = line.to_string();
        if let Some(directive) = trimmed.strip_prefix("&TestsFile:") {
            if found_declaration || awaiting_test {
                return Err(format!(
                    "{}:{}: TestsFile directives must appear before test declarations",
                    path.display(),
                    index + 1
                ));
            }
            match directive {
                "Parallel" => {
                    if options.parallel.is_some() {
                        return Err(format!(
                            "{}:{}: duplicate Parallel directive",
                            path.display(),
                            index + 1
                        ));
                    }
                    options.parallel = Some(
                        thread::available_parallelism()
                            .map(usize::from)
                            .unwrap_or(1),
                    );
                }
                "StopOnFailed" => {
                    if options.stop_on_failed != StopOnFailed::Disabled {
                        return Err(format!(
                            "{}:{}: duplicate StopOnFailed directive",
                            path.display(),
                            index + 1
                        ));
                    }
                    options.stop_on_failed = StopOnFailed::StopStarted;
                }
                "StopOnFailed:DontStopStarted" => {
                    if options.stop_on_failed != StopOnFailed::Disabled {
                        return Err(format!(
                            "{}:{}: duplicate StopOnFailed directive",
                            path.display(),
                            index + 1
                        ));
                    }
                    options.stop_on_failed = StopOnFailed::DontStopStarted;
                }
                _ if directive.starts_with("Parallel[") && directive.ends_with(']') => {
                    if options.parallel.is_some() {
                        return Err(format!(
                            "{}:{}: duplicate Parallel directive",
                            path.display(),
                            index + 1
                        ));
                    }
                    let limit = &directive[9..directive.len() - 1];
                    let limit = limit.parse::<usize>().map_err(|_| {
                        format!(
                            "{}:{}: Parallel limit must be a positive integer",
                            path.display(),
                            index + 1
                        )
                    })?;
                    if limit == 0 {
                        return Err(format!(
                            "{}:{}: Parallel limit must be greater than zero",
                            path.display(),
                            index + 1
                        ));
                    }
                    options.parallel = Some(limit);
                }
                _ => {
                    return Err(format!(
                        "{}:{}: invalid TestsFile directive '{directive}'",
                        path.display(),
                        index + 1
                    ));
                }
            }
            continue;
        }
        if trimmed == "#[test]" {
            if awaiting_test {
                return Err(format!(
                    "{}:{}: duplicate #[test] attribute",
                    path.display(),
                    index + 1
                ));
            }
            awaiting_test = true;
            found_declaration = true;
            continue;
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            found_declaration = true;
        }
        if awaiting_test && !trimmed.is_empty() && !trimmed.starts_with("//") {
            let rest = trimmed.strip_prefix("fun ").ok_or_else(|| {
                format!(
                    "{}:{}: #[test] must annotate a function",
                    path.display(),
                    index + 1
                )
            })?;
            let name = rest.split('(').next().unwrap_or_default().trim();
            if name.is_empty()
                || !name.bytes().enumerate().all(|(position, byte)| {
                    if position == 0 {
                        byte.is_ascii_alphabetic() || byte == b'_'
                    } else {
                        byte.is_ascii_alphanumeric() || byte == b'_'
                    }
                })
            {
                return Err(format!(
                    "{}:{}: invalid test function name",
                    path.display(),
                    index + 1
                ));
            }
            if !rest[name.len()..].trim_start().starts_with("()") {
                return Err(format!(
                    "{}:{}: test functions cannot have parameters",
                    path.display(),
                    index + 1
                ));
            }
            if tests.iter().any(|test: &TestDefinition| test.name == name) {
                return Err(format!(
                    "{}:{}: duplicate test '{name}'",
                    path.display(),
                    index + 1
                ));
            }
            tests.push(TestDefinition { name: name.into() });
            awaiting_test = false;
            output_line = line.replacen("()", "() result:None", 1);
        }
        output.push_str(&output_line);
        output.push('\n');
    }
    if awaiting_test {
        return Err(format!(
            "{}: #[test] must annotate a function",
            path.display()
        ));
    }
    if tests.is_empty() {
        return Err(format!("{}: no #[test] functions found", path.display()));
    }
    Ok((output, tests, options))
}

#[cfg(test)]
mod test_directive_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn project(source: &str) -> PathBuf {
        let root = env::temp_dir().join(format!(
            "adamantium-test-directives-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("code")).unwrap();
        fs::write(root.join("code/tests.ad"), source).unwrap();
        root
    }

    #[test]
    fn parses_parallel_and_stop_directives() {
        let root = project(
            "&TestsFile:Parallel[3]\n&TestsFile:StopOnFailed:DontStopStarted\n#[test]\nfun works() {}\n",
        );
        let (_, tests, options) = test_source(&root).unwrap();
        assert_eq!(tests.len(), 1);
        assert_eq!(options.parallel, Some(3));
        assert_eq!(options.stop_on_failed, StopOnFailed::DontStopStarted);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_invalid_test_file_directives() {
        for directive in [
            "&TestsFile:Parallel[0]",
            "&TestsFile:Parallel[no]",
            "&TestsFile:Unknown",
        ] {
            let root = project(&format!("{directive}\n#[test]\nfun works() {{}}\n"));
            assert!(test_source(&root).is_err(), "{directive}");
            fs::remove_dir_all(root).unwrap();
        }
    }
}

fn test_program(
    root: &Path,
    tests_source: &str,
    test: &TestDefinition,
) -> Result<(PathBuf, String, typed::Program), String> {
    let (root, project_name, mut sources, bindings, professional) = project_sources(root)?;
    let main = sources
        .iter_mut()
        .find(|(module, _)| module.is_empty())
        .expect("main module is always loaded");
    main.1.push('\n');
    main.1.push_str(tests_source);
    main.1.push_str(&format!(
        "\nfun __adamantium_test_entry() result:None {{ {}(); }}\n",
        test.name
    ));
    let program = analyze_sources(&root, &sources, bindings, professional)?;
    Ok((root, project_name, program))
}

fn list_tests(root: &Path) -> Result<(), String> {
    let canonical = root
        .canonicalize()
        .map_err(|error| format!("{}: {error}", root.display()))?;
    let (source, tests, _) = test_source(&canonical)?;
    for test in &tests {
        test_program(&canonical, &source, test)?;
        println!("{}", test.name);
    }
    Ok(())
}

fn run_tests(root: &Path, filter: Option<&str>, verbose: bool) -> Result<ExitCode, String> {
    let canonical = root
        .canonicalize()
        .map_err(|error| format!("{}: {error}", root.display()))?;
    let (source, all_tests, options) = test_source(&canonical)?;
    let tests = all_tests
        .into_iter()
        .filter(|test| filter.is_none_or(|filter| test.name == filter))
        .collect::<Vec<_>>();
    if tests.is_empty() {
        return Err(filter.map_or_else(
            || "no tests found".into(),
            |name| format!("test '{name}' was not found"),
        ));
    }
    println!(
        "running {} test{}",
        tests.len(),
        if tests.len() == 1 { "" } else { "s" }
    );
    let mut prepared = VecDeque::new();
    for (index, test) in tests.iter().enumerate() {
        let (root, project_name, program) = test_program(&canonical, &source, test)?;
        let output_name = format!("{}_test_{index}", project_name);
        let executable = emit_executable(
            &root,
            &output_name,
            &program,
            "__adamantium_test_entry",
            &output_name,
            optimizer::Level::default(),
        )?;
        prepared.push_back((index, test.clone(), root, executable));
    }
    let worker_limit = match options.stop_on_failed {
        StopOnFailed::StopStarted => 1,
        _ => options.parallel.unwrap_or(1),
    }
    .max(1)
    .min(prepared.len());
    let queue = Arc::new(Mutex::new(prepared));
    let results = Arc::new(Mutex::new(
        (0..tests.len()).map(|_| None).collect::<Vec<_>>(),
    ));
    let stop = Arc::new(AtomicBool::new(false));
    thread::scope(|scope| {
        for _ in 0..worker_limit {
            let queue = Arc::clone(&queue);
            let results = Arc::clone(&results);
            let stop = Arc::clone(&stop);
            scope.spawn(move || {
                loop {
                    if stop.load(Ordering::Acquire)
                        && options.stop_on_failed != StopOnFailed::Disabled
                    {
                        break;
                    }
                    let Some((index, test, root, executable)) = queue.lock().unwrap().pop_front()
                    else {
                        break;
                    };
                    let result = Command::new(executable).current_dir(root).output();
                    if result.as_ref().is_ok_and(|output| !output.status.success())
                        || result.is_err()
                    {
                        stop.store(true, Ordering::Release);
                    }
                    results.lock().unwrap()[index] = Some((test, result));
                }
            });
        }
    });
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    for result in Arc::into_inner(results).unwrap().into_inner().unwrap() {
        let Some((test, output)) = result else {
            skipped += 1;
            continue;
        };
        let output =
            output.map_err(|error| format!("could not run test '{}': {error}", test.name))?;
        if output.status.success() {
            passed += 1;
            println!("test {} ... ok", test.name);
        } else {
            failed += 1;
            println!("test {} ... FAILED", test.name);
        }
        if verbose || !output.status.success() {
            if !output.stdout.is_empty() {
                print!("{}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprint!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
    }
    if skipped != 0 {
        println!("{skipped} test(s) not run after the first failure");
    }
    println!(
        "\ntest result: {}. {passed} passed; {failed} failed",
        if failed == 0 { "ok" } else { "FAILED" }
    );
    Ok(if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn create_project(root: &Path) -> Result<(), String> {
    adamantium_project::create(root)
}

fn current_directory() -> Result<OsString, String> {
    env::current_dir()
        .map(Into::into)
        .map_err(|e| format!("could not read the current directory: {e}"))
}

fn no_more_args(mut args: impl Iterator<Item = OsString>) -> Result<(), String> {
    if args.next().is_some() {
        Err("too many arguments; use --help".into())
    } else {
        Ok(())
    }
}

fn build(root: &Path, level: optimizer::Level) -> Result<PathBuf, String> {
    let (root, name, statements) = analyze(root)?;
    emit_executable(&root, &name, &statements, "main", &name, level)
}

fn emit_executable(
    root: &Path,
    project_name: &str,
    statements: &typed::Program,
    entry: &str,
    output_name: &str,
    level: optimizer::Level,
) -> Result<PathBuf, String> {
    let target = root.join("target");
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    let asm = target.join(format!("{output_name}.asm"));
    let obj = target.join(if cfg!(target_os = "linux") {
        format!("{output_name}.o")
    } else {
        format!("{output_name}.obj")
    });
    let exe = target.join(if cfg!(target_os = "linux") {
        output_name.to_string()
    } else {
        format!("{output_name}.exe")
    });
    let runtime = target.join("adamantium_runtime.lib");
    let runtime_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/runtime.lib"));
    if runtime_bytes.is_empty() {
        return Err(
            "building Adamantium executables is supported on Windows and Linux x86-64".into(),
        );
    }
    fs::write(&runtime, runtime_bytes).map_err(|e| e.to_string())?;
    let optimized = optimizer::optimize(statements.clone(), entry, level);
    fs::write(
        &asm,
        codegen::assembly_entry(&optimized, entry, level == optimizer::Level::O2),
    )
    .map_err(|e| e.to_string())?;
    let nasm = nasm_command();
    execute(
        Command::new(nasm)
            .arg("-f")
            .arg(if cfg!(target_os = "linux") {
                "elf64"
            } else {
                "win64"
            })
            .arg(&asm)
            .arg("-o")
            .arg(&obj),
        "NASM",
    )?;
    link(&target, project_name, &obj, &runtime, &exe)?;
    Ok(exe)
}

fn check(root: &Path) -> Result<(), String> {
    analyze(root).map(|_| ())
}

fn doctor(root: &Path) -> Result<ExitCode, String> {
    println!("Adamantium doctor {}", env!("CARGO_PKG_VERSION"));
    let mut failures = 0usize;
    doctor_result(
        cfg!(all(
            any(target_os = "windows", target_os = "linux"),
            target_arch = "x86_64"
        )),
        format!("platform: {}-{}", env::consts::OS, env::consts::ARCH),
        "use a supported Windows or Linux x86-64 build of Adamantium",
        &mut failures,
    );
    doctor_result(
        !include_bytes!(concat!(env!("OUT_DIR"), "/runtime.lib")).is_empty(),
        "embedded runtime: available".into(),
        "reinstall Adamantium because this executable has no embedded runtime",
        &mut failures,
    );
    doctor_result(
        adamantium_compiler::architecture_smoke_test("fun main").is_ok(),
        "compiler frontend: operational".into(),
        "reinstall Adamantium because the compiler self-check failed",
        &mut failures,
    );

    let nasm = nasm_command();
    let nasm_check = command_summary(&nasm, &["--version"]);
    doctor_result(
        nasm_check.is_some(),
        format!(
            "NASM: {}",
            nasm_check.unwrap_or_else(|| format!("not found ({})", nasm.to_string_lossy()))
        ),
        "install NASM, use the portable distribution, or set ADAMANTIUM_NASM",
        &mut failures,
    );

    let (linker, linker_arguments) = linker_probe();
    let linker_check = command_summary(&linker, &linker_arguments);
    doctor_result(
        linker_check.is_some(),
        format!(
            "linker: {}",
            linker_check.unwrap_or_else(|| format!("not found ({})", linker.to_string_lossy()))
        ),
        "use the portable distribution, install platform linker tools, or set ADAMANTIUM_LINKER",
        &mut failures,
    );

    if root.join("project.toml").is_file() {
        match analyze(root) {
            Ok(_) => println!("[ok] project and package configuration: {}", root.display()),
            Err(error) => {
                failures += 1;
                println!("[error] project or package configuration: {error}");
                println!(
                    "        help: fix project.toml, requirement.toml, and reported source errors"
                );
            }
        }
    } else {
        println!(
            "[skip] project configuration: no project.toml in {}",
            root.display()
        );
    }

    if failures == 0 {
        println!("Doctor found no problems.");
        Ok(ExitCode::SUCCESS)
    } else {
        println!("Doctor found {failures} blocking problem(s).");
        Ok(ExitCode::FAILURE)
    }
}

fn doctor_result(success: bool, message: String, help: &str, failures: &mut usize) {
    if success {
        println!("[ok] {message}");
    } else {
        *failures += 1;
        println!("[error] {message}");
        println!("        help: {help}");
    }
}

fn nasm_command() -> OsString {
    env::var_os("ADAMANTIUM_NASM").unwrap_or_else(|| {
        let bundled = portable_tools_directory()
            .map(|tools| tools.join(if cfg!(windows) { "nasm.exe" } else { "nasm" }));
        if let Some(bundled) = bundled
            && bundled.is_file()
        {
            return bundled.into_os_string();
        }
        let installed = PathBuf::from(
            env::var_os("ProgramFiles").unwrap_or_else(|| "C:\\Program Files".into()),
        )
        .join("NASM/nasm.exe");
        if installed.is_file() {
            installed.into_os_string()
        } else {
            "nasm".into()
        }
    })
}

fn linker_probe() -> (OsString, Vec<&'static str>) {
    if let Some(linker) = env::var_os("ADAMANTIUM_LINKER") {
        return (linker, vec!["--version"]);
    }
    if let Some(tools) = portable_tools_directory() {
        let bundled = if cfg!(windows) {
            tools.join("lld-link.exe")
        } else {
            tools.join("zig/zig")
        };
        if bundled.is_file() {
            return (bundled.into_os_string(), vec!["--version"]);
        }
    }
    if cfg!(windows) {
        ("link.exe".into(), vec!["/?"])
    } else {
        ("cc".into(), vec!["--version"])
    }
}

fn command_summary(command: &OsString, arguments: &[&str]) -> Option<String> {
    let output = Command::new(command).args(arguments).output().ok()?;
    let text = if output.stdout.is_empty() {
        &output.stderr
    } else {
        &output.stdout
    };
    let first_line = String::from_utf8_lossy(text)
        .lines()
        .next()
        .unwrap_or("available")
        .trim()
        .to_owned();
    Some(if first_line.is_empty() {
        "available".into()
    } else {
        first_line
    })
}

fn clean_project(root: &Path) -> Result<(), String> {
    let requested_root = root;
    let root = requested_root
        .canonicalize()
        .map_err(|error| format!("{}: {error}", requested_root.display()))?;
    read_toml(&root.join("project.toml"))?;
    let target = root.join("target");
    if !target.exists() {
        println!("Project is already clean");
        return Ok(());
    }
    let metadata = fs::symlink_metadata(&target)
        .map_err(|error| format!("could not inspect {}: {error}", target.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "refusing to clean symbolic link {}",
            target.display()
        ));
    }
    if target.parent() != Some(root.as_path())
        || target.file_name().is_none_or(|name| name != "target")
    {
        return Err("refusing to clean a target outside the project root".into());
    }
    fs::remove_dir_all(&target)
        .map_err(|error| format!("could not clean {}: {error}", target.display()))?;
    println!("Cleaned {}", target.display());
    Ok(())
}

fn format_project(root: &Path) -> Result<usize, String> {
    let requested_root = root;
    let root = requested_root
        .canonicalize()
        .map_err(|e| format!("{}: {e}", requested_root.display()))?;
    if let Err(errors) = adamantium_project::read_manifest(&root) {
        return Err(diagnostics::multiple_errors(errors));
    }
    let mut files = Vec::new();
    collect_ad_files(&root.join("code"), &mut files)?;
    let mut pending_writes = Vec::new();
    for path in files {
        let original = fs::read_to_string(&path)
            .map_err(|error| format!("could not read {}: {error}", path.display()))?;
        let formatted = formatter::format_source(&original)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if formatted != original {
            pending_writes.push((path, formatted));
        }
    }
    let changed = pending_writes.len();
    for (path, formatted) in pending_writes {
        fs::write(&path, formatted)
            .map_err(|error| format!("could not write {}: {error}", path.display()))?;
        println!("Formatted {}", display_path(&path));
    }
    Ok(changed)
}

fn display_path(path: &Path) -> String {
    let display = path.display().to_string();
    display.strip_prefix(r"\\?\").unwrap_or(&display).to_owned()
}

fn collect_ad_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut entries = fs::read_dir(directory)
        .map_err(|e| format!("could not read {}: {e}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("could not read {}: {e}", directory.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|e| format!("could not inspect {}: {e}", path.display()))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_ad_files(&path, files)?;
        } else if file_type.is_file() && path.extension().is_some_and(|extension| extension == "ad")
        {
            files.push(path);
        }
    }
    Ok(())
}

fn packages(table: &toml::Table) -> Result<Vec<Package>, String> {
    let Some(values) = table.get("packages").and_then(toml::Value::as_table) else {
        return Err("requirement.toml: expected a [packages] table".into());
    };
    if table.keys().any(|key| key != "packages") {
        return Err("requirement.toml: only the [packages] table is supported".into());
    }
    let mut result = Vec::new();
    for (source, value) in values {
        let version = value.as_str().ok_or_else(|| {
            format!("requirement.toml: package '{source}' version must be a string")
        })?;
        let requirement = adamantium_packages::Requirement::new(source, version)
            .map_err(|error| format!("requirement.toml: package '{source}': {error}"))?;
        result.push(Package {
            name: requirement.name,
            source: requirement.source,
            version: requirement.version.to_string(),
        });
    }
    result.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(result)
}

fn is_official_package_source(source: &str) -> bool {
    source
        .strip_prefix("https://github.com/")
        .and_then(|path| path.split_once('/'))
        .is_some_and(|(owner, _)| {
            owner.eq_ignore_ascii_case("AdmerPRO") || owner.eq_ignore_ascii_case("AdamantiumORG")
        })
}

fn community_package_warning(package: &Package) -> Option<String> {
    (!is_official_package_source(&package.source)).then(|| {
        format!(
            "warning: package '{}' is a community package and is not controlled by Adamantium",
            package.name
        )
    })
}

fn install_packages(root: &Path) -> Result<(), String> {
    let requested_root = root;
    let root = requested_root
        .canonicalize()
        .map_err(|error| format!("{}: {error}", requested_root.display()))?;
    let requirements = read_toml(&root.join("requirement.toml"))?;
    let requested = packages(&requirements)?;
    if requested.is_empty() {
        println!("No packages to install");
        return Ok(());
    }
    let downloader = env::var_os("ADAMANTIUM_CURL").unwrap_or_else(|| {
        if cfg!(windows) {
            "curl.exe".into()
        } else {
            "curl".into()
        }
    });
    let mut manifests = std::collections::BTreeMap::new();
    for package in &requested {
        collect_package_manifests(&root, package, &downloader, &mut manifests)?;
    }
    let roots = requested
        .iter()
        .map(|package| adamantium_packages::Requirement::new(&package.source, &package.version))
        .collect::<Result<Vec<_>, _>>()?;
    let lock = adamantium_packages::resolve(&roots, &manifests)?;
    for locked in &lock.packages {
        let repository_name = adamantium_packages::github_repository_name(&locked.source)?;
        let package = Package {
            name: repository_name,
            source: locked.source.clone(),
            version: locked.version.clone(),
        };
        if let Some(warning) = community_package_warning(&package) {
            eprintln!("{warning}");
        }
        let cache = package_cache_directory(&root, &package)?;
        fs::create_dir_all(&cache)
            .map_err(|error| format!("could not create {}: {error}", cache.display()))?;
        let cached_wasm = cache.join("adamantium_packet.wasm");
        if package.version == "nightly" || !valid_wasm_file(&cached_wasm) {
            let temporary = cache.join("adamantium_packet.wasm.download");
            let tag = package_release_tag(&package.version);
            let url = format!(
                "{}/releases/download/{tag}/adamantium_packet.wasm",
                package.source
            );
            download_package_file(&downloader, &url, &temporary, &package)?;
            if !valid_wasm_file(&temporary) {
                let _ = fs::remove_file(&temporary);
                return Err(format!(
                    "package '{}' did not contain a valid WebAssembly binary",
                    package.name
                ));
            }
            replace_file(&temporary, &cached_wasm)?;
        }
        let directory = root
            .join("packages")
            .join(&package.name)
            .join(&package.version);
        fs::create_dir_all(&directory)
            .map_err(|error| format!("could not create {}: {error}", directory.display()))?;
        let destination = directory.join("adamantium_packet.wasm");
        let manifest_destination = directory.join("adamantium_packet.toml");
        fs::copy(&cached_wasm, &destination)
            .map_err(|error| format!("could not install {}: {error}", destination.display()))?;
        fs::copy(cache.join("adamantium_packet.toml"), &manifest_destination).map_err(|error| {
            format!(
                "could not install {}: {error}",
                manifest_destination.display()
            )
        })?;
        println!("Installed {} {}", package.name, package.version);
    }
    fs::write(root.join("adamantium.lock"), lock.render()?)
        .map_err(|error| format!("could not write adamantium.lock: {error}"))?;
    println!(
        "Installed {} package{}",
        lock.packages.len(),
        if lock.packages.len() == 1 { "" } else { "s" }
    );
    Ok(())
}

fn prepare_package_release(root: &Path) -> Result<adamantium_packages::ReleaseBundle, String> {
    adamantium_packages::generate_release(
        &root.join(adamantium_packages::PACKAGE_MANIFEST),
        &root.join(adamantium_packages::PACKAGE_WASM),
        &root.join("target/package-release"),
    )
}

fn publish_package_release(root: &Path) -> Result<(), String> {
    let bundle = prepare_package_release(root)?;
    let exists = Command::new("gh")
        .args(["release", "view", &bundle.tag])
        .current_dir(root)
        .status()
        .map_err(|error| format!("could not run GitHub CLI: {error}"))?
        .success();
    let mut command = Command::new("gh");
    command.current_dir(root);
    if exists {
        command.args(["release", "upload", &bundle.tag]);
        command.args(&bundle.assets).arg("--clobber");
    } else {
        command.args(["release", "create", &bundle.tag]);
        command
            .args(&bundle.assets)
            .args(["--generate-notes", "--target", "HEAD"]);
    }
    execute(&mut command, "GitHub Release publishing")?;
    println!("Published {}", bundle.tag);
    Ok(())
}

fn collect_package_manifests(
    root: &Path,
    package: &Package,
    downloader: &std::ffi::OsStr,
    manifests: &mut std::collections::BTreeMap<String, adamantium_packages::Manifest>,
) -> Result<(), String> {
    if manifests.contains_key(&package.source) {
        return Ok(());
    }
    let cache = package_cache_directory(root, package)?;
    fs::create_dir_all(&cache)
        .map_err(|error| format!("could not create {}: {error}", cache.display()))?;
    let manifest_path = cache.join("adamantium_packet.toml");
    let manifest = match fs::read_to_string(&manifest_path)
        .ok()
        .and_then(|source| adamantium_packages::Manifest::parse(&source).ok())
        .filter(|manifest| {
            package.version != "nightly" && manifest.package.version == package.version
        }) {
        Some(manifest) => {
            packages::validate_manifest(&manifest_path, &package.version)?;
            manifest
        }
        None => {
            let temporary = cache.join("adamantium_packet.toml.download");
            let tag = package_release_tag(&package.version);
            let url = format!(
                "{}/releases/download/{tag}/adamantium_packet.toml",
                package.source
            );
            download_package_file(downloader, &url, &temporary, package)?;
            let source = fs::read_to_string(&temporary)
                .map_err(|error| format!("could not read package manifest: {error}"))?;
            let manifest = adamantium_packages::Manifest::parse(&source)?;
            if package.version != "nightly" && manifest.package.version != package.version {
                return Err(format!(
                    "package '{}' manifest declares version {}, expected {}",
                    package.name, manifest.package.version, package.version
                ));
            }
            packages::validate_manifest(&temporary, &package.version)?;
            replace_file(&temporary, &manifest_path)?;
            manifest
        }
    };
    manifests.insert(package.source.clone(), manifest.clone());
    for (source, version) in &manifest.dependencies {
        let dependency = adamantium_packages::Requirement::new(source, version)?;
        collect_package_manifests(
            root,
            &Package {
                name: dependency.name,
                source: dependency.source,
                version: dependency.version.to_string(),
            },
            downloader,
            manifests,
        )?;
    }
    Ok(())
}

fn package_release_tag(version: &str) -> String {
    if version == "nightly" {
        "adamantium_packet_nightly".into()
    } else {
        format!("adamantium_packet_{}", version.replace('.', "_"))
    }
}

fn package_cache_directory(root: &Path, package: &Package) -> Result<PathBuf, String> {
    let repository = package
        .source
        .strip_prefix("https://github.com/")
        .and_then(|path| path.split_once('/'))
        .ok_or_else(|| format!("invalid package source '{}'", package.source))?;
    Ok(root
        .join("packages")
        .join(".cache")
        .join(repository.0)
        .join(repository.1.trim_end_matches(".git"))
        .join(&package.version))
}

fn valid_wasm_file(path: &Path) -> bool {
    fs::read(path).is_ok_and(|bytes| adamantium_wasm::validate_package(&bytes).is_ok())
}

fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    if destination.exists() {
        fs::remove_file(destination)
            .map_err(|error| format!("could not replace {}: {error}", destination.display()))?;
    }
    fs::rename(source, destination)
        .map_err(|error| format!("could not install {}: {error}", destination.display()))
}

fn download_package_file(
    downloader: &std::ffi::OsStr,
    url: &str,
    destination: &Path,
    package: &Package,
) -> Result<(), String> {
    let status = Command::new(downloader)
        .args([
            "-fL",
            "--proto",
            "=https",
            "--proto-redir",
            "=https",
            "--output",
        ])
        .arg(destination)
        .arg(url)
        .status()
        .map_err(|error| format!("could not download package '{}': {error}", package.name))?;
    if !status.success() {
        let _ = fs::remove_file(destination);
        return Err(format!(
            "failed to download package '{}' version {} from {url}",
            package.name, package.version
        ));
    }
    Ok(())
}

fn analyze(root: &Path) -> Result<(PathBuf, String, typed::Program), String> {
    let (root, name, sources, bindings, professional) = project_sources(root)?;
    let statements = analyze_sources(&root, &sources, bindings, professional)?;
    Ok((root, name, statements))
}

fn project_sources(root: &Path) -> Result<ProjectSources, String> {
    let requested_root = root;
    let root = requested_root
        .canonicalize()
        .map_err(|e| format!("{}: {e}", requested_root.display()))?;
    let manifest = adamantium_project::read_manifest(&root);
    let mut errors = manifest.as_ref().err().cloned().unwrap_or_default();
    let requirements = read_toml(&root.join("requirement.toml"))?;
    let packages = match packages(&requirements) {
        Ok(packages) => packages,
        Err(error) => {
            errors.push(error);
            Vec::new()
        }
    };
    if !errors.is_empty() {
        return Err(diagnostics::multiple_errors(errors));
    }
    let manifest = manifest.expect("validated project manifest");
    let packages = packages_from_lock(&root, packages)?;
    let mut sources = load_modules(&root.join("code"))?;
    if manifest.professional {
        for (module, source) in &sources {
            syntax::validate_professional(source).map_err(|error| {
                let file = if module.is_empty() { "main" } else { module };
                format!(
                    "{}:{}",
                    root.join(format!("code/{file}.ad")).display(),
                    error
                )
            })?;
        }
    }
    let bindings = packages::load_bindings(&root, &packages, &mut sources)?;
    Ok((
        root,
        manifest.name,
        sources,
        bindings,
        manifest.professional,
    ))
}

fn packages_from_lock(root: &Path, direct: Vec<Package>) -> Result<Vec<Package>, String> {
    let path = root.join("adamantium.lock");
    if !path.exists() {
        return Ok(direct);
    }
    let source = fs::read_to_string(&path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let lock = adamantium_packages::Lockfile::parse(&source)?;
    for requirement in &direct {
        if !lock.packages.iter().any(|package| {
            package.source == requirement.source && package.version == requirement.version
        }) {
            return Err(format!(
                "adamantium.lock is out of date for package '{}'; run 'adamantium install'",
                requirement.name
            ));
        }
    }
    lock.packages
        .into_iter()
        .map(|package| {
            Ok(Package {
                name: adamantium_packages::github_repository_name(&package.source)?,
                source: package.source,
                version: package.version,
            })
        })
        .collect()
}

fn analyze_sources(
    root: &Path,
    sources: &[(String, String)],
    bindings: Vec<packages::Binding>,
    professional: bool,
) -> Result<typed::Program, String> {
    let source_path = root.join("code/main.ad");
    let parsed = syntax::parse_modules_with_mode(sources, professional)
        .map_err(|e| format!("{}:{e}", source_path.display()))?;
    let main_source = sources
        .iter()
        .find(|(name, _)| name.is_empty())
        .map_or("", |(_, source)| source.as_str());
    let mut statements = typed::check_diagnostic(&parsed, main_source).map_err(|diagnostic| {
        diagnostics::render_diagnostic(&diagnostic, &source_path, main_source)
    })?;
    statements.package_functions = bindings
        .into_iter()
        .map(|binding| (binding.canonical, binding.function))
        .collect();
    for warning in diagnostics::warnings(&parsed) {
        eprintln!("{}:{warning}", source_path.display());
    }
    Ok(statements)
}

fn load_modules(code: &Path) -> Result<Vec<(String, String)>, String> {
    adamantium_project::load_modules(code, |source| {
        syntax::module_dependencies(source).map_err(|error| error.to_string())
    })
}

fn link(target: &Path, name: &str, obj: &Path, runtime: &Path, exe: &Path) -> Result<(), String> {
    if cfg!(target_os = "linux") {
        let libraries = include_str!(concat!(env!("OUT_DIR"), "/runtime-libraries.txt"));
        let configured = env::var_os("ADAMANTIUM_LINKER");
        let bundled = portable_tools_directory()
            .map(|tools| tools.join("zig/zig"))
            .filter(|linker| linker.is_file());
        let use_bundled_zig = configured.is_none() && bundled.is_some();
        let mut command = Command::new(
            configured
                .or_else(|| bundled.clone().map(PathBuf::into_os_string))
                .unwrap_or_else(|| "cc".into()),
        );
        if use_bundled_zig {
            command.arg("cc");
        }
        command.arg("-no-pie").arg(obj).arg(runtime);
        command
            .args(linux_runtime_libraries(libraries))
            .arg("-o")
            .arg(exe);
        return execute(&mut command, "Linux C linker");
    }
    let mut arguments = vec![
        OsString::from("/nologo"),
        OsString::from("/machine:x64"),
        OsString::from("/subsystem:console"),
        OsString::from("/dynamicbase"),
        OsString::from("/nxcompat"),
        format!("/out:{}", exe.display()).into(),
        obj.as_os_str().into(),
        runtime.as_os_str().into(),
    ];
    let auxiliary = extract_runtime_auxiliary_libraries(target)?;
    arguments.extend(
        include_str!(concat!(env!("OUT_DIR"), "/runtime-libraries.txt"))
            .split_whitespace()
            .map(|library| {
                auxiliary
                    .iter()
                    .find(|path| path.file_name().is_some_and(|name| name == library))
                    .map_or_else(|| OsString::from(library), |path| path.as_os_str().into())
            }),
    );
    arguments.push("kernel32.lib".into());

    if let Some(linker) = env::var_os("ADAMANTIUM_LINKER") {
        return execute(
            Command::new(linker).args(&arguments),
            "Microsoft linker configured by ADAMANTIUM_LINKER",
        );
    }
    if let Some(tools) = portable_tools_directory() {
        let linker = tools.join("lld-link.exe");
        if linker.is_file() {
            arguments.push(format!("/libpath:{}", tools.join("lib").display()).into());
            return execute(Command::new(linker).args(&arguments), "bundled LLVM linker");
        }
    }
    if env::var_os("VSCMD_ARG_TGT_ARCH").is_some() {
        return execute(
            Command::new("link.exe").args(&arguments),
            "Microsoft linker",
        );
    }

    let vcvars = find_vcvars64().ok_or(
        "could not find Visual Studio C++ build tools; install the MSVC x64 tools or run from an x64 Native Tools Command Prompt",
    )?;
    let response = target.join(format!("{name}.link.rsp"));
    let response_text = arguments
        .iter()
        .map(|argument| format!("\"{}\"", argument.to_string_lossy().replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&response, response_text).map_err(|e| format!("{}: {e}", response.display()))?;
    execute(
        Command::new("cmd.exe")
            .args(["/d", "/c", "call"])
            .arg(&vcvars)
            .args([">", "nul", "&&", "link.exe"])
            .arg(format!("@{}", response.display())),
        "Microsoft linker through the Visual Studio x64 environment",
    )
}

fn linux_runtime_libraries(libraries: &str) -> Vec<&str> {
    libraries.split_whitespace().collect()
}

#[cfg(test)]
mod linker_tests {
    use super::linux_runtime_libraries;

    #[test]
    fn linux_linker_keeps_rust_runtime_libraries() {
        let libraries = "-lgcc_s -lutil -lrt -lpthread -lm -ldl -lc";
        assert_eq!(
            linux_runtime_libraries(libraries),
            [
                "-lgcc_s",
                "-lutil",
                "-lrt",
                "-lpthread",
                "-lm",
                "-ldl",
                "-lc"
            ]
        );
    }
}

fn portable_tools_directory() -> Option<PathBuf> {
    env::current_exe()
        .ok()
        .and_then(|executable| executable.parent().map(|parent| parent.join("tools")))
}

fn extract_runtime_auxiliary_libraries(target: &Path) -> Result<Vec<PathBuf>, String> {
    let bundle = include_bytes!(concat!(env!("OUT_DIR"), "/runtime-auxiliary-libraries.bin"));
    let mut cursor = 0;
    let mut paths = Vec::new();
    while cursor < bundle.len() {
        if bundle.len() - cursor < 12 {
            return Err("embedded runtime library bundle is invalid".into());
        }
        let name_length =
            u32::from_le_bytes(bundle[cursor..cursor + 4].try_into().unwrap()) as usize;
        cursor += 4;
        let data_length =
            u64::from_le_bytes(bundle[cursor..cursor + 8].try_into().unwrap()) as usize;
        cursor += 8;
        if bundle.len() - cursor < name_length + data_length {
            return Err("embedded runtime library bundle is invalid".into());
        }
        let name = std::str::from_utf8(&bundle[cursor..cursor + name_length])
            .map_err(|_| "embedded runtime library name is invalid")?;
        cursor += name_length;
        let path = target.join(name);
        fs::write(&path, &bundle[cursor..cursor + data_length])
            .map_err(|error| format!("{}: {error}", path.display()))?;
        cursor += data_length;
        paths.push(path);
    }
    Ok(paths)
}

fn find_vcvars64() -> Option<PathBuf> {
    let program_files_x86 = env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files (x86)"));
    let vswhere = program_files_x86.join("Microsoft Visual Studio/Installer/vswhere.exe");
    if let Ok(output) = Command::new(vswhere)
        .args([
            "-latest",
            "-products",
            "*",
            "-requires",
            "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
            "-property",
            "installationPath",
        ])
        .output()
        && output.status.success()
        && let Ok(installation) = String::from_utf8(output.stdout)
    {
        let candidate = PathBuf::from(installation.trim()).join("VC/Auxiliary/Build/vcvars64.bat");
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let program_files = env::var_os("ProgramFiles")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Program Files"));
    for year in ["2022", "2019"] {
        for edition in ["Community", "Professional", "Enterprise", "BuildTools"] {
            let candidate = program_files
                .join("Microsoft Visual Studio")
                .join(year)
                .join(edition)
                .join("VC/Auxiliary/Build/vcvars64.bat");
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

fn read_toml(path: &Path) -> Result<toml::Table, String> {
    adamantium_project::read_toml(path)
}

fn execute(command: &mut Command, label: &str) -> Result<(), String> {
    let status = command
        .status()
        .map_err(|e| format!("could not run {label}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} failed ({status})"))
    }
}

#[cfg(test)]
mod package_tests {
    use super::*;

    #[test]
    fn parses_github_wasm_packages_and_versions() {
        let table = r#"[packages]
"https://github.com/AdmerPRO/Math" = "1.2.3"
"https://github.com/community/text-tools" = "0.4.0"
"https://github.com/AdamantiumORG/Json" = "nightly"
"#
        .parse::<toml::Table>()
        .unwrap();
        assert_eq!(
            packages(&table).unwrap(),
            [
                Package {
                    name: "Json".into(),
                    source: "https://github.com/AdamantiumORG/Json".into(),
                    version: "nightly".into(),
                },
                Package {
                    name: "Math".into(),
                    source: "https://github.com/AdmerPRO/Math".into(),
                    version: "1.2.3".into(),
                },
                Package {
                    name: "text-tools".into(),
                    source: "https://github.com/community/text-tools".into(),
                    version: "0.4.0".into(),
                },
            ]
        );
    }

    #[test]
    fn identifies_official_package_owners() {
        assert!(is_official_package_source(
            "https://github.com/AdmerPRO/Math"
        ));
        assert!(is_official_package_source(
            "https://github.com/AdamantiumORG/Math"
        ));
        assert!(!is_official_package_source(
            "https://github.com/community/Math"
        ));
    }

    #[test]
    fn warns_about_community_packages() {
        let community = Package {
            name: "some.package".into(),
            source: "https://github.com/community/some.package".into(),
            version: "1.0.0".into(),
        };
        assert_eq!(
            community_package_warning(&community).as_deref(),
            Some(
                "warning: package 'some.package' is a community package and is not controlled by Adamantium"
            )
        );

        let official = Package {
            name: "Math".into(),
            source: "https://github.com/AdamantiumORG/Math".into(),
            version: "1.0.0".into(),
        };
        assert_eq!(community_package_warning(&official), None);
    }

    #[test]
    fn rejects_invalid_package_sources_and_versions() {
        for manifest in [
            "[packages]\n\"https://gitlab.com/Other/Name\"=\"1.0.0\"",
            "[packages]\n\"https://github.com/Owner/Name/extra\"=\"1.0.0\"",
            "[packages]\n\"https://github.com/AdmerPRO/../Name\"=\"1.0.0\"",
            "[packages]\n\"https://github.com/AdmerPRO/Name\"=\"latest\"",
        ] {
            assert!(packages(&manifest.parse().unwrap()).is_err(), "{manifest}");
        }
    }

    #[test]
    fn selects_the_nightly_release_tag() {
        assert_eq!(package_release_tag("nightly"), "adamantium_packet_nightly");
        assert_eq!(package_release_tag("1.2.3"), "adamantium_packet_1_2_3");
    }
}

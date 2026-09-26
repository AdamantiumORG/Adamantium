use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT_PROJECT: AtomicUsize = AtomicUsize::new(0);

struct Project(PathBuf);

impl Project {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "adamantium-memory-safety-{}-{}",
            std::process::id(),
            NEXT_PROJECT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("code")).unwrap();
        fs::write(
            root.join("project.toml"),
            "name='MemorySafety'\nversion='1.0.0'\ndescription=''\nauthors=[]\n",
        )
        .unwrap();
        fs::write(root.join("requirement.toml"), "[packages]\n").unwrap();
        fs::write(root.join("code/main.ad"), source).unwrap();
        Self(root)
    }

    fn check(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_adamantium"))
            .arg("check")
            .arg(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_valid(source: &str) {
    let output = Project::new(source).check();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_invalid(source: &str, expected: &str) {
    let output = Project::new(source).check();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "program unexpectedly passed");
    assert!(
        stderr.contains(expected),
        "missing '{expected}' in:\n{stderr}"
    );
}

#[test]
fn aliases_keep_storage_alive_until_the_last_reachable_name_is_removed() {
    assert_valid(
        "fun main() { var root=10; var alias=root.as_variable; alias.detach(); root.remove; alias.sync(); print.newline(alias); }",
    );
    assert_invalid(
        "fun main() { var root=10; var alias=root.as_variable; root.remove; alias.remove; print.newline(alias); }",
        "variable 'alias' was removed",
    );
}

#[test]
fn offsets_cannot_outlive_removed_targets() {
    assert_invalid(
        "fun main() { var source=10; var address=source.offset; source.remove; print.newline(address.by_offset); }",
        "offset target was removed",
    );
    assert_invalid(
        "fun main() { var source=10; var address=source.offset; var copy=address; source.remove; print.newline(copy.by_offset); }",
        "offset target was removed",
    );
}

#[test]
fn removed_and_renamed_bindings_cannot_be_reused() {
    assert_invalid(
        "fun main() { var value=10; value.remove; value=20; }",
        "variable 'value' was removed",
    );
    assert_invalid(
        "fun main() { var value=10; value.changename(saved); print.newline(value); }",
        "variable 'value' was removed",
    );
}

#[test]
fn dereference_requires_a_live_typed_offset() {
    assert_invalid(
        "fun main() { var value=10; print.newline(value.by_offset); }",
        "by_offset requires an offset",
    );
    assert_invalid(
        "fun main() { var value=10; var first=value.offset; var second=first.offset; }",
        "offsets of offsets are not supported",
    );
}

#[test]
fn offsets_cannot_escape_through_collections() {
    assert_invalid(
        "fun main() { var value=10; var address=value.offset; var escaped=List[address]; }",
        "offset values cannot be stored in Lists",
    );
}

#[test]
fn class_lifecycle_hooks_reject_immediate_recursion() {
    assert_invalid(
        "class Item(pub value:int) { fun __new__() {} fun __change__() { self.value=2; } } fun main() { var item=Item(value=1); }",
        "__change__ cannot modify self recursively",
    );
    assert_invalid(
        "class Item(pub value:int) { fun __new__() {} fun __remove__() { self.remove; } } fun main() { var item=Item(value=1); }",
        "__remove__ cannot remove an object recursively",
    );
}

#[test]
fn nested_scope_bindings_do_not_escape_or_destroy_shadowed_parents() {
    assert_valid(
        "fun main() { var value=10; if true { var value=20; value.remove; } print.newline(value); }",
    );
    assert_invalid(
        "fun main() { if true { var local=10; } print.newline(local); }",
        "variable 'local' is out of scope",
    );
}

use std::collections::{BTreeSet, HashMap};

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

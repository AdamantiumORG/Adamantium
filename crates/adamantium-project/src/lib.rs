use std::path::PathBuf;

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
            Err(adamantium_diagnostics::Diagnostic::error(
                "E300",
                "missing code/main.ad",
                Default::default(),
            ))
        }
    }
}

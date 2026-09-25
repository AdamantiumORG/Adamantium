//! Process execution without invoking a command shell.

use crate::{Error, Result};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Output {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Command {
    program: String,
    arguments: Vec<String>,
    directory: Option<PathBuf>,
    environment: Vec<(String, String)>,
}

impl Command {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            ..Self::default()
        }
    }

    pub fn argument(mut self, value: impl Into<String>) -> Self {
        self.arguments.push(value.into());
        self
    }

    pub fn directory(mut self, value: impl Into<PathBuf>) -> Self {
        self.directory = Some(value.into());
        self
    }

    pub fn environment(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.push((name.into(), value.into()));
        self
    }

    pub fn run(&self) -> Result<Output> {
        if self.program.is_empty() {
            return Err(Error::new("process.run", "program cannot be empty"));
        }
        let mut command = std::process::Command::new(&self.program);
        command
            .args(&self.arguments)
            .envs(self.environment.iter().cloned());
        if let Some(directory) = &self.directory {
            command.current_dir(directory);
        }
        let output = command
            .output()
            .map_err(|error| Error::new("process.run", error))?;
        Ok(Output {
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_empty_program() {
        assert_eq!(Command::new("").run().unwrap_err().operation, "process.run");
    }
}

//! A small task abstraction backed by native threads.

use crate::{Error, Result};

#[derive(Debug)]
pub struct Task<T>(Option<std::thread::JoinHandle<T>>);

impl<T: Send + 'static> Task<T> {
    pub fn spawn(function: impl FnOnce() -> T + Send + 'static) -> Self {
        Self(Some(std::thread::spawn(function)))
    }

    pub fn join(mut self) -> Result<T> {
        self.0
            .take()
            .expect("a task owns exactly one join handle")
            .join()
            .map_err(|_| Error::new("async.join", "task panicked"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_returns_its_value() {
        assert_eq!(Task::spawn(|| 21 * 2).join().unwrap(), 42);
    }
}

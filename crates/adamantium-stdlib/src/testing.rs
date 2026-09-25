//! Assertions used by the Adamantium test runner and standard-library tests.

use std::fmt::Debug;

pub fn equal<T: PartialEq + Debug>(actual: &T, expected: &T) -> std::result::Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!("expected {expected:?}, found {actual:?}"))
    }
}

pub fn truthy(value: bool, message: impl Into<String>) -> std::result::Result<(), String> {
    value.then_some(()).ok_or_else(|| message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assertion_failures_are_readable() {
        assert!(equal(&1, &1).is_ok());
        assert_eq!(equal(&1, &2).unwrap_err(), "expected 2, found 1");
        assert_eq!(truthy(false, "failed").unwrap_err(), "failed");
    }
}

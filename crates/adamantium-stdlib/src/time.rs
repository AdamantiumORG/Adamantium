//! Wall-clock, monotonic timing, and sleeping.

use crate::{Error, Result};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub fn unix_milliseconds() -> Result<u64> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| Error::new("time.unix_milliseconds", error))?;
    u64::try_from(duration.as_millis()).map_err(|error| Error::new("time.unix_milliseconds", error))
}

pub fn sleep_milliseconds(milliseconds: u64) {
    std::thread::sleep(Duration::from_millis(milliseconds));
}

#[derive(Clone, Copy, Debug)]
pub struct Stopwatch(Instant);

impl Stopwatch {
    pub fn start() -> Self {
        Self(Instant::now())
    }

    pub fn elapsed_milliseconds(self) -> u64 {
        u64::try_from(self.0.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_time_is_after_the_project_epoch() {
        assert!(unix_milliseconds().unwrap() > 1_700_000_000_000);
    }
}

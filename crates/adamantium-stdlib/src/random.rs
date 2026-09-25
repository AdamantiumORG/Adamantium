//! Dependency-free deterministic and process-seeded random numbers.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static STATE: AtomicU64 = AtomicU64::new(0);

fn initial_seed() -> u64 {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos() as u64);
    nanos ^ u64::from(std::process::id()) ^ 0x9e37_79b9_7f4a_7c15
}

pub fn seed(value: u64) {
    STATE.store(value.max(1), Ordering::Relaxed);
}

pub fn next_u64() -> u64 {
    let mut current = STATE.load(Ordering::Relaxed);
    if current == 0 {
        current = initial_seed();
    }
    loop {
        let mut next = current;
        next ^= next << 13;
        next ^= next >> 7;
        next ^= next << 17;
        match STATE.compare_exchange_weak(current, next, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return next,
            Err(observed) => current = observed,
        }
    }
}

pub fn range(start: u64, end: u64) -> Option<u64> {
    let width = end.checked_sub(start)?;
    (width != 0).then(|| start + next_u64() % width)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_sequence_is_repeatable_and_ranges_are_bounded() {
        seed(42);
        let first = next_u64();
        seed(42);
        assert_eq!(next_u64(), first);
        for _ in 0..100 {
            assert!((5..9).contains(&range(5, 9).unwrap()));
        }
        assert_eq!(range(1, 1), None);
    }
}

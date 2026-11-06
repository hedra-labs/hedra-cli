//! The doubling wait between retries that every sandbox retry loop takes,
//! each with its own first wait and cap.

use std::time::Duration;

/// A wait that starts at `first` and doubles after each retry up to `cap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backoff {
    first: Duration,
    cap: Duration,
    next: Duration,
}

impl Backoff {
    pub const fn new(first: Duration, cap: Duration) -> Self {
        Self {
            first,
            cap,
            next: first,
        }
    }

    /// The wait before the next retry.
    pub fn wait(&self) -> Duration {
        self.next
    }

    /// Doubles the next wait, up to the cap.
    pub fn grow(&mut self) {
        self.next = (self.next * 2).min(self.cap);
    }

    /// Starts again from the first wait, once a retry got through.
    pub fn reset(&mut self) {
        self.next = self.first;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wait_doubles_to_its_cap_and_resets_to_its_first() {
        let mut b = Backoff::new(Duration::from_millis(500), Duration::from_secs(3));
        let mut seen = Vec::new();
        for _ in 0..5 {
            seen.push(b.wait().as_millis());
            b.grow();
        }
        assert_eq!(seen, [500, 1000, 2000, 3000, 3000]);
        b.reset();
        assert_eq!(b.wait(), Duration::from_millis(500));
    }
}

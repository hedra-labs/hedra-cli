//! The client for a sandbox's streaming calls.

use std::time::Duration;

/// How often the client pings an idle HTTP/2 connection.
pub const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

#[cfg(test)]
mod tests {
    use super::*;

    /// The broker's keepalive enforcement minimum (its `MinTime`): a client
    /// that pings more often than this has its connection closed.
    const BROKER_KEEPALIVE_MIN_TIME: Duration = Duration::from_secs(10);

    #[test]
    fn pings_are_spaced_above_the_brokers_min_time() {
        assert!(KEEPALIVE_INTERVAL > BROKER_KEEPALIVE_MIN_TIME);
    }
}

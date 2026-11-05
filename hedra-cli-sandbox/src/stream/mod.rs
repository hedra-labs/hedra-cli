//! The client for a sandbox's streaming calls: the broker connection, the
//! calls over it, and the broker's statuses.

use std::time::Duration;

pub mod calls;
pub mod spool;
pub mod status;
pub mod transport;

/// How often the client pings an idle HTTP/2 connection.
pub const KEEPALIVE_INTERVAL: Duration = Duration::from_secs(30);

/// How long a ping may go unanswered before the connection is dropped, so a
/// dead network ends a session within about 40 s.
pub const KEEPALIVE_TIMEOUT: Duration = Duration::from_secs(10);

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

//! The bytes held for local sockets that have not taken them yet, per
//! connection and for the invocation.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tokio::sync::Notify;

use super::Limits;

pub(crate) struct Budget {
    per_connection: usize,
    total_max: usize,
    inner: Mutex<Inner>,
}

#[derive(Default)]
struct Inner {
    next: u64,
    total: usize,
    held: HashMap<u64, (usize, Arc<Notify>)>,
}

impl Budget {
    pub(crate) fn new(limits: Limits) -> Self {
        Self {
            per_connection: limits.connection_buffer,
            total_max: limits.total_buffer,
            inner: Mutex::new(Inner::default()),
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// A new connection's id, and the signal that resets it.
    pub(crate) fn register(&self) -> (u64, Arc<Notify>) {
        let mut g = self.lock();
        let id = g.next;
        g.next += 1;
        let reset = Arc::new(Notify::new());
        g.held.insert(id, (0, reset.clone()));
        (id, reset)
    }

    pub(crate) fn unregister(&self, id: u64) {
        let mut g = self.lock();
        if let Some((n, _)) = g.held.remove(&id) {
            g.total -= n;
        }
    }

    /// Holds `n` more bytes for `id`. When its buffer or the invocation's
    /// would overflow, the connection holding the most is reset: false when
    /// that is `id` itself, and the others are signalled.
    pub(crate) fn take(&self, id: u64, n: usize) -> bool {
        let mut g = self.lock();
        loop {
            let Some(&(mine, _)) = g.held.get(&id) else {
                return false;
            };
            if mine + n <= self.per_connection && g.total + n <= self.total_max {
                break;
            }
            let victim = g
                .held
                .iter()
                .map(|(k, (held, _))| (*k, if *k == id { held + n } else { *held }))
                .max_by_key(|(k, held)| (*held, *k == id))
                .map(|(k, _)| k)
                .expect("id is held");
            let (held, reset) = g.held.remove(&victim).expect("present");
            g.total -= held;
            if victim == id {
                return false;
            }
            reset.notify_one();
        }
        let entry = g.held.get_mut(&id).expect("checked");
        entry.0 += n;
        g.total += n;
        true
    }

    /// `n` of `id`'s bytes reached its socket.
    pub(crate) fn release(&self, id: u64, n: usize) {
        let mut g = self.lock();
        if let Some(entry) = g.held.get_mut(&id) {
            let n = n.min(entry.0);
            entry.0 -= n;
            g.total -= n;
        }
    }

    #[cfg(test)]
    pub(crate) fn total(&self) -> usize {
        self.lock().total
    }

    #[cfg(test)]
    pub(crate) fn connections(&self) -> usize {
        self.lock().held.len()
    }

    #[cfg(test)]
    pub(crate) fn held(&self, id: u64) -> Option<usize> {
        self.lock().held.get(&id).map(|(n, _)| *n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn budget(per: usize, total: usize) -> Budget {
        Budget::new(Limits {
            max_connections: 90,
            connection_buffer: per,
            total_buffer: total,
        })
    }

    #[test]
    fn a_full_connection_is_reset() {
        let b = budget(100, 1000);
        let (a, _) = b.register();
        assert!(b.take(a, 60));
        assert!(!b.take(a, 60));
        assert_eq!(b.total(), 0);
    }

    #[test]
    fn a_full_budget_resets_the_largest() {
        let b = budget(100, 150);
        let (a, ra) = b.register();
        let (c, _) = b.register();
        assert!(b.take(a, 90));
        assert!(b.take(c, 40));
        // c is smaller, so a is reset and c keeps its bytes.
        assert!(b.take(c, 40));
        assert_eq!(b.held(a), None);
        assert_eq!(b.held(c), Some(80));
        assert_eq!(b.total(), 80);
        // The signal was stored for a.
        let notified = ra.notified();
        tokio::pin!(notified);
        assert!(futures_util::FutureExt::now_or_never(notified).is_some());
    }

    /// A connection that closes with bytes still held gives them back.
    #[test]
    fn a_closed_connection_gives_its_bytes_back() {
        let b = budget(100, 150);
        let (a, _) = b.register();
        assert!(b.take(a, 60));
        b.unregister(a);
        assert_eq!(b.total(), 0);
        assert_eq!(b.connections(), 0);
    }

    /// On a tie, the connection whose bytes overflowed the budget is the
    /// one reset, and the others keep what they hold. Every one of 32
    /// connections takes a turn at overflowing, so the tie-break, not the
    /// map's iteration order, has to pick it.
    #[test]
    fn a_tie_resets_the_connection_that_overflowed() {
        const N: u64 = 32;
        for over in 0..N {
            let b = budget(100, 30 * N as usize - 5);
            let signals: Vec<_> = (0..N).map(|_| b.register()).collect();
            for (id, _) in &signals {
                assert!(b.take(*id, if *id == over { 20 } else { 30 }));
            }
            // `over` would hold 30, as every other one does.
            assert!(!b.take(over, 10), "{over}");
            assert_eq!(b.held(over), None, "{over}");
            assert_eq!(b.total(), 30 * (N as usize - 1), "{over}");
            for (id, reset) in &signals {
                if *id == over {
                    continue;
                }
                assert_eq!(b.held(*id), Some(30), "{over}: {id}");
                let notified = reset.notified();
                tokio::pin!(notified);
                assert!(
                    futures_util::FutureExt::now_or_never(notified).is_none(),
                    "{over}: {id} was signalled"
                );
            }
        }
    }
}

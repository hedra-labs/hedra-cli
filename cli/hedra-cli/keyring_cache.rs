//! Process-lifetime memo in front of the OS credential store.
//!
//! Nothing in the SDK caches a keyring read. `AuthCredentialSource::Keyring`
//! calls `active_store().get(..)` inline every time it resolves, and the
//! resolve sites multiply:
//!
//! * `RoutingAuthProvider::apply` resolves once through `has_credentials()`
//!   to pick a satisfiable security requirement, throws the value away, then
//!   resolves again inside the scheme provider's `apply` — two reads per
//!   request.
//! * `build_http_request` (which is where `apply` runs) sits inside *both*
//!   the pagination loop and the retry loop, so that pair is multiplied by
//!   pages and by attempts.
//! * `auth login` round-trips the freshly written OAuth bundle back out of
//!   the store twice in the same process — once to print the token claims,
//!   once for the key bootstrap — because `LoginFlow::run` returns `()` and
//!   cannot hand the bundle over in memory.
//!
//! On macOS every one of those is a `SecKeychainFindGenericPassword` against
//! the login keychain. A CLI invocation runs one command against one
//! credential, so one read should cover it.
//!
//! **Write-through, not invalidate-on-write.** A `set` populates the entry
//! rather than dropping it. That is what makes the login round-trips free —
//! invalidating would merely buy the reads straight back.
//!
//! Correctness rests on the process being the only writer for its own
//! lifetime, which holds: the CLI is short-lived and single-command. A
//! concurrent `hedra-cli` in another process could write behind us, but it
//! could equally write one instruction after our uncached read — there is no
//! window here that did not already exist.
//!
//! This layer does **not** reduce the number of distinct Keychain *items*
//! touched, and so does not by itself reduce the number of authorization
//! prompts — macOS authorizes per item, not per read. It removes redundant
//! work. Reducing the item count is [`super::active_key`]'s job, and making
//! the remaining prompts stop recurring on every upgrade needs a stable code
//! signature, which is neither of these.
//!
//! Hand-written and .fernignore-protected — the generator never emits this
//! file; the ignore entry is what stops regeneration from deleting it.
//! Declared from `custom.rs` via `#[path]`, like `auth.rs` and
//! `workspaces.rs`.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use fern_cli_sdk::auth::KeyringStore;
use fern_cli_sdk::error::CliError;

/// Wrap `inner` in a [`CachingKeyringStore`].
///
/// The caller installs the result — see `custom::register`, which stacks the
/// `KeyAuth` projection on top of this.
pub(crate) fn memoize(inner: Arc<dyn KeyringStore>) -> Arc<dyn KeyringStore> {
    Arc::new(CachingKeyringStore::new(inner))
}

/// Read-through, write-through memo over another [`KeyringStore`].
///
/// `None` is cached as a real answer: a missing entry is the common case for
/// an unbound scheme, and re-asking the OS about it on every resolve is the
/// same wasted syscall as re-asking about a present one.
///
/// Errors are never cached. A transient backend failure must not poison the
/// entry for the rest of the process.
pub(crate) struct CachingKeyringStore {
    inner: Arc<dyn KeyringStore>,
    entries: RwLock<HashMap<(String, String), Option<String>>>,
}

// Hand-written rather than derived: the map holds credentials in plaintext,
// and the `KeyringStore` trait requires `Debug`. A derived impl would spill
// every cached secret into any `{:?}` — including panic messages and
// `tracing` fields.
//
// The inner store is rendered by its `backend_label()` rather than its own
// `Debug`, so this guarantee does not depend on the wrapped implementation
// keeping its values out of `{:?}`. `MockKeyringStore` derives `Debug` over
// its value map and does exactly that.
impl std::fmt::Debug for CachingKeyringStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachingKeyringStore")
            .field("backend", &self.inner.backend_label())
            .field("cached_entries", &self.len())
            .finish()
    }
}

impl CachingKeyringStore {
    pub(crate) fn new(inner: Arc<dyn KeyringStore>) -> Self {
        Self {
            inner,
            entries: RwLock::new(HashMap::new()),
        }
    }

    fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    /// Cached lookup. `Some(hit)` means the answer is known — including
    /// `Some(None)`, "the store definitively has no such entry".
    fn cached(&self, key: &(String, String)) -> Option<Option<String>> {
        // A poisoned lock means a panic happened while the map was held.
        // Recover the map rather than propagating: this sits in the auth
        // path, and a cache is never worth failing a command over.
        let guard = match self.entries.read() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.get(key).cloned()
    }

    fn store(&self, key: (String, String), value: Option<String>) {
        let mut guard = match self.entries.write() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let _ = guard.insert(key, value);
    }

    /// Drop what we think we know. Used when a write fails: the backend's
    /// true state is no longer something we can infer, so the next read must
    /// go and ask.
    fn forget(&self, key: &(String, String)) {
        let mut guard = match self.entries.write() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        let _ = guard.remove(key);
    }
}

impl KeyringStore for CachingKeyringStore {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, CliError> {
        let key = (service.to_string(), account.to_string());
        if let Some(hit) = self.cached(&key) {
            return Ok(hit);
        }
        // Deliberately not holding a lock across the backend call — on macOS
        // this can block on an interactive authorization dialog. Two threads
        // racing the same cold key would each ask the OS once; that costs a
        // duplicate read, not a wrong answer, and the CLI resolves
        // credentials sequentially anyway.
        tracing::trace!(service, account, "credential cache miss; reading backend");
        let value = self.inner.get(service, account)?;
        self.store(key, value.clone());
        Ok(value)
    }

    fn set(&self, service: &str, account: &str, value: &str) -> Result<(), CliError> {
        let key = (service.to_string(), account.to_string());
        match self.inner.set(service, account, value) {
            Ok(()) => {
                self.store(key, Some(value.to_string()));
                Ok(())
            }
            Err(e) => {
                self.forget(&key);
                Err(e)
            }
        }
    }

    fn delete(&self, service: &str, account: &str) -> Result<(), CliError> {
        let key = (service.to_string(), account.to_string());
        match self.inner.delete(service, account) {
            Ok(()) => {
                self.store(key, None);
                Ok(())
            }
            Err(e) => {
                self.forget(&key);
                Err(e)
            }
        }
    }

    fn backend_label(&self) -> String {
        // The cache is not a storage location. `auth status` must keep
        // reporting where credentials actually live.
        self.inner.backend_label()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Inner store that counts backend hits, so a test can assert the cache
    /// actually stopped one. `fail_writes` drives the invalidation paths.
    #[derive(Debug, Default)]
    struct CountingStore {
        values: RwLock<HashMap<(String, String), String>>,
        gets: AtomicUsize,
        sets: AtomicUsize,
        deletes: AtomicUsize,
        fail_writes: bool,
        fail_reads: bool,
    }

    impl CountingStore {
        fn seeded(pairs: &[(&str, &str, &str)]) -> Arc<Self> {
            let store = Self::default();
            {
                let mut guard = store.values.write().unwrap();
                for (service, account, value) in pairs {
                    let _ = guard.insert(
                        (service.to_string(), account.to_string()),
                        value.to_string(),
                    );
                }
            }
            Arc::new(store)
        }

        fn failing_writes() -> Arc<Self> {
            Arc::new(Self {
                fail_writes: true,
                ..Self::default()
            })
        }

        fn failing_reads() -> Arc<Self> {
            Arc::new(Self {
                fail_reads: true,
                ..Self::default()
            })
        }

        fn gets(&self) -> usize {
            self.gets.load(Ordering::SeqCst)
        }
    }

    impl KeyringStore for CountingStore {
        fn get(&self, service: &str, account: &str) -> Result<Option<String>, CliError> {
            let _ = self.gets.fetch_add(1, Ordering::SeqCst);
            if self.fail_reads {
                return Err(CliError::Auth("backend down".to_string()));
            }
            Ok(self
                .values
                .read()
                .unwrap()
                .get(&(service.to_string(), account.to_string()))
                .cloned())
        }

        fn set(&self, service: &str, account: &str, value: &str) -> Result<(), CliError> {
            let _ = self.sets.fetch_add(1, Ordering::SeqCst);
            if self.fail_writes {
                return Err(CliError::Auth("backend refused the write".to_string()));
            }
            let _ = self.values.write().unwrap().insert(
                (service.to_string(), account.to_string()),
                value.to_string(),
            );
            Ok(())
        }

        fn delete(&self, service: &str, account: &str) -> Result<(), CliError> {
            let _ = self.deletes.fetch_add(1, Ordering::SeqCst);
            if self.fail_writes {
                return Err(CliError::Auth("backend refused the delete".to_string()));
            }
            let _ = self
                .values
                .write()
                .unwrap()
                .remove(&(service.to_string(), account.to_string()));
            Ok(())
        }

        fn backend_label(&self) -> String {
            "counting".to_string()
        }
    }

    // The headline: the double resolve in `RoutingAuthProvider::apply`, and
    // its multiplication by pages and retries, collapses to one backend read.
    #[test]
    fn repeated_reads_hit_the_backend_once() {
        let inner = CountingStore::seeded(&[("hedra-cli", "KeyAuth", "kid:secret")]);
        let cache = CachingKeyringStore::new(inner.clone());

        for _ in 0..10 {
            assert_eq!(
                cache.get("hedra-cli", "KeyAuth").unwrap().as_deref(),
                Some("kid:secret")
            );
        }

        assert_eq!(
            inner.gets(),
            1,
            "every read after the first must be served from the memo"
        );
    }

    // A missing entry is an answer worth remembering: an unbound scheme is
    // probed on every resolve just like a bound one.
    #[test]
    fn absence_is_cached_too() {
        let inner = CountingStore::seeded(&[]);
        let cache = CachingKeyringStore::new(inner.clone());

        assert!(cache.get("hedra-cli", "OAuth").unwrap().is_none());
        assert!(cache.get("hedra-cli", "OAuth").unwrap().is_none());

        assert_eq!(inner.gets(), 1, "a known-absent entry must not be re-asked");
    }

    // Write-through is the point: `auth login` writes the OAuth bundle and
    // then reads it straight back, twice, in the same process.
    #[test]
    fn write_through_serves_the_read_back() {
        let inner = CountingStore::seeded(&[]);
        let cache = CachingKeyringStore::new(inner.clone());

        cache
            .set("hedra-cli", "OAuth", r#"{"access_token":"jwt"}"#)
            .unwrap();
        assert_eq!(
            cache.get("hedra-cli", "OAuth").unwrap().as_deref(),
            Some(r#"{"access_token":"jwt"}"#)
        );
        assert_eq!(
            cache.get("hedra-cli", "OAuth").unwrap().as_deref(),
            Some(r#"{"access_token":"jwt"}"#)
        );

        assert_eq!(
            inner.gets(),
            0,
            "a value we just wrote must not be read back from the backend"
        );
    }

    #[test]
    fn delete_is_remembered_as_absence() {
        let inner = CountingStore::seeded(&[("hedra-cli", "OAuth", "stale")]);
        let cache = CachingKeyringStore::new(inner.clone());

        cache.delete("hedra-cli", "OAuth").unwrap();

        assert!(cache.get("hedra-cli", "OAuth").unwrap().is_none());
        assert_eq!(
            inner.gets(),
            0,
            "a delete we performed is enough to know it is gone"
        );
    }

    // A failed write leaves the backend in a state we cannot infer, so the
    // memo must step aside rather than serve a guess.
    #[test]
    fn a_failed_write_invalidates_rather_than_lying() {
        let inner = CountingStore::failing_writes();
        let cache = CachingKeyringStore::new(inner.clone());

        // Prime the memo with a known-absent answer, then fail a write over it.
        assert!(cache.get("hedra-cli", "KeyAuth").unwrap().is_none());
        assert_eq!(inner.gets(), 1);
        assert!(cache.set("hedra-cli", "KeyAuth", "kid:secret").is_err());

        assert!(cache.get("hedra-cli", "KeyAuth").unwrap().is_none());
        assert_eq!(
            inner.gets(),
            2,
            "after a failed write the next read must go to the backend"
        );
    }

    #[test]
    fn a_failed_delete_invalidates_too() {
        let inner = CountingStore::failing_writes();
        let cache = CachingKeyringStore::new(inner.clone());

        assert!(cache.get("hedra-cli", "OAuth").unwrap().is_none());
        assert!(cache.delete("hedra-cli", "OAuth").is_err());
        assert!(cache.get("hedra-cli", "OAuth").unwrap().is_none());

        assert_eq!(
            inner.gets(),
            2,
            "a failed delete must not be recorded as absence"
        );
    }

    // A backend hiccup must not become a permanent "no credentials" for the
    // rest of the process.
    #[test]
    fn errors_are_not_cached() {
        let inner = CountingStore::failing_reads();
        let cache = CachingKeyringStore::new(inner.clone());

        assert!(cache.get("hedra-cli", "KeyAuth").is_err());
        assert!(cache.get("hedra-cli", "KeyAuth").is_err());

        assert_eq!(
            inner.gets(),
            2,
            "an errored read must be retried, not memoised"
        );
    }

    // The four hedra-cli slots share a service name; only the account differs.
    #[test]
    fn entries_are_keyed_by_service_and_account() {
        let inner = CountingStore::seeded(&[
            ("hedra-cli", "KeyAuth", "kid:secret"),
            ("hedra-cli", "OAuth", "bundle"),
            ("other-cli", "KeyAuth", "someone-elses"),
        ]);
        let cache = CachingKeyringStore::new(inner.clone());

        assert_eq!(
            cache.get("hedra-cli", "KeyAuth").unwrap().as_deref(),
            Some("kid:secret")
        );
        assert_eq!(
            cache.get("hedra-cli", "OAuth").unwrap().as_deref(),
            Some("bundle")
        );
        assert_eq!(
            cache.get("other-cli", "KeyAuth").unwrap().as_deref(),
            Some("someone-elses")
        );
        assert_eq!(inner.gets(), 3, "three distinct keys, three backend reads");

        assert_eq!(
            cache.get("hedra-cli", "KeyAuth").unwrap().as_deref(),
            Some("kid:secret")
        );
        assert_eq!(inner.gets(), 3, "and none of them re-read");
    }

    // `auth status` prints where credentials live; the cache is not a place.
    #[test]
    fn backend_label_passes_through() {
        let cache = CachingKeyringStore::new(CountingStore::seeded(&[]));
        assert_eq!(cache.backend_label(), "counting");
    }

    // The trait requires Debug and the map holds plaintext credentials.
    // `CountingStore` derives Debug over its values on purpose: the
    // guarantee has to hold even when the wrapped store is careless, which
    // `MockKeyringStore` is.
    #[test]
    fn debug_does_not_leak_cached_secrets() {
        let inner = CountingStore::seeded(&[("hedra-cli", "KeyAuth", "kid:super-secret")]);
        let cache = CachingKeyringStore::new(inner);
        let _ = cache.get("hedra-cli", "KeyAuth").unwrap();

        let rendered = format!("{cache:?}");
        assert!(
            !rendered.contains("super-secret"),
            "Debug must not render cached credentials: {rendered}"
        );
        assert!(
            rendered.contains("cached_entries"),
            "unexpected shape: {rendered}"
        );
    }
}

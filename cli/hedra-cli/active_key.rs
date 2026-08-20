//! Serves the `KeyAuth` credential slot from the workspace key map, so the
//! active API key is stored once instead of twice.
//!
//! The SDK's data-plane auth reads from a *fixed address*.
//! `inject_keyring_sources` walks the auth bindings and appends
//! `AuthCredentialSource::keyring(cli_name, scheme_name)` to each token
//! chain — for this CLI that is `(hedra-cli, "KeyAuth")`, from the
//! `BearerAuth::new("KeyAuth")` in the generated `main.rs`. It cannot be told
//! "read the workspace map and pick the active entry", and the binding
//! cannot be replaced from `custom.rs`: `CliApp::auth()` appends rather than
//! replaces, and a duplicate binding — though it would win, since
//! `build_provider_with_strategy` keeps the last provider registered under a
//! name — is still walked by `auth status`, which would then print the
//! scheme twice, once stale and once empty.
//!
//! So the address stays; what backs it changes. Every keyring read in the
//! process funnels through `active_store()`, which makes the store the one
//! place that can answer that address from somewhere else.
//!
//! Before this, `workspaces::activate` and the bootstrap mints each wrote
//! the active credential into its own `KeyAuth` item — a verbatim copy of an
//! entry already in the map. That cost a second keychain item (its own
//! authorization prompt, and its own write on every `select`), and the two
//! copies could disagree: a `map.save()` that succeeded followed by a
//! `set(KeyAuth)` that failed left `auth status` reporting no credential
//! while a perfectly good key sat in the map.
//!
//! ## Precedence
//!
//! The map wins when it has an answer; a real item is the fallback. That
//! ordering is deliberate:
//!
//! * A leftover item from a release that still wrote one is frozen at
//!   whichever key was active at upgrade time. If it won, `workspaces
//!   select` would silently keep serving the old workspace's key.
//! * `auth login --with-token` (`run_token_paste`) writes the item directly.
//!   A user who has never logged in has no map, so the fallback serves it.
//!   A user who has both has expressed two intents; the map — which
//!   `workspaces select` maintains — is the more recent one, and
//!   `HEDRA_API_KEY` still shadows everything either way.
//!
//! [`super::auth::drop_stale_key_mirror`] deletes the leftover item at the
//! moments the active credential changes, so an upgraded install converges
//! on one item rather than carrying an unmaintained credential forever.
//!
//! Hand-written and .fernignore-protected — the generator never emits this
//! file; the ignore entry is what stops regeneration from deleting it.

use std::sync::Arc;

use fern_cli_sdk::auth::KeyringStore;
use fern_cli_sdk::error::CliError;

use super::auth::KEY_SCHEME;
use super::workspaces::{WorkspaceKeyMap, WORKSPACE_KEYS_SCHEME};

/// Wrap `inner` so reads of the `KeyAuth` slot are answered from the
/// workspace key map.
///
/// Belongs *outside* the memo, not inside it: the projection resolves
/// through whatever it wraps, so wrapping the cache means the workspace-map
/// read it performs is itself cached. The other order would memoise a
/// derived `KeyAuth` value that a later write to the map could not
/// invalidate.
pub(crate) fn project(inner: Arc<dyn KeyringStore>) -> Arc<dyn KeyringStore> {
    Arc::new(ActiveKeyStore { inner })
}

#[derive(Debug)]
pub(crate) struct ActiveKeyStore {
    inner: Arc<dyn KeyringStore>,
}

impl ActiveKeyStore {
    /// Read the map through `inner` and hand back the active credential.
    ///
    /// Deliberately not `WorkspaceKeyMap::load`, which resolves through
    /// `active_store()` — that is *this* store, and re-entering it to answer
    /// a question about itself is a loop waiting to be closed by a future
    /// edit. Going straight to `inner` also keeps the read to one hop.
    ///
    /// A map that will not parse is treated as absent, matching `load`'s own
    /// `unwrap_or_default`: a corrupt map should degrade to "no credential",
    /// which the auth path already reports well, not to a hard error from
    /// inside a credential lookup.
    fn active_credential(&self, service: &str) -> Result<Option<String>, CliError> {
        let Some(raw) = self.inner.get(service, WORKSPACE_KEYS_SCHEME)? else {
            return Ok(None);
        };
        let Ok(map) = serde_json::from_str::<WorkspaceKeyMap>(&raw) else {
            return Ok(None);
        };
        Ok(map.active_credential().map(str::to_string))
    }
}

impl KeyringStore for ActiveKeyStore {
    fn get(&self, service: &str, account: &str) -> Result<Option<String>, CliError> {
        if account != KEY_SCHEME {
            return self.inner.get(service, account);
        }
        if let Some(credential) = self.active_credential(service)? {
            return Ok(Some(credential));
        }
        // No map answer: fall back to a real item, which is what a
        // `--with-token` paste (or a not-yet-migrated install) leaves.
        self.inner.get(service, account)
    }

    // `set` and `delete` pass through untouched. Nothing in this CLI writes
    // the `KeyAuth` slot any more — `record_key` and `activate` maintain the
    // map instead — so the only writers left are the SDK's own token-paste
    // and logout paths, and both mean exactly what they say.
    fn set(&self, service: &str, account: &str, value: &str) -> Result<(), CliError> {
        self.inner.set(service, account, value)
    }

    fn delete(&self, service: &str, account: &str) -> Result<(), CliError> {
        self.inner.delete(service, account)
    }

    fn backend_label(&self) -> String {
        self.inner.backend_label()
    }
}

/// Install the projection over an in-memory store, for tests that assert on
/// the `KeyAuth` slot. Returns the mock so a test can still seed and inspect
/// raw items beneath the projection.
#[cfg(test)]
pub(crate) fn projected_mock() -> Arc<fern_cli_sdk::auth::MockKeyringStore> {
    let mock = Arc::new(fern_cli_sdk::auth::MockKeyringStore::new());
    fern_cli_sdk::auth::set_active_store(project(mock.clone()));
    mock
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::custom::workspaces::HeldKey;
    use fern_cli_sdk::auth::MockKeyringStore;
    use std::collections::BTreeMap;

    const CLI: &str = "test-cli";

    fn write_map(store: &MockKeyringStore, map: &WorkspaceKeyMap) {
        store
            .set(
                CLI,
                WORKSPACE_KEYS_SCHEME,
                &serde_json::to_string(map).unwrap(),
            )
            .unwrap();
    }

    fn held(key_id: &str, credential: &str) -> HeldKey {
        HeldKey {
            key_id: key_id.to_string(),
            credential: credential.to_string(),
            workspace_name: None,
            expires_at: None,
        }
    }

    fn bound(workspace: &str, key_id: &str, credential: &str) -> WorkspaceKeyMap {
        let mut keys = BTreeMap::new();
        let _ = keys.insert(workspace.to_string(), held(key_id, credential));
        WorkspaceKeyMap {
            active_workspace_id: Some(workspace.to_string()),
            keys,
            unbound_key: None,
        }
    }

    #[test]
    fn key_auth_is_served_from_the_active_workspace_entry() {
        let mock = Arc::new(MockKeyringStore::new());
        write_map(&mock, &bound("w1", "key_1", "key_1:secret"));
        let store = ActiveKeyStore { inner: mock };

        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("key_1:secret"),
            "no KeyAuth item exists; it must come from the map"
        );
    }

    // Selecting a workspace is only a marker move — the projection is what
    // turns that into a different credential on the wire.
    #[test]
    fn moving_the_active_marker_changes_the_served_key() {
        let mock = Arc::new(MockKeyringStore::new());
        let mut map = bound("w1", "key_1", "key_1:secret");
        let _ = map
            .keys
            .insert("w2".to_string(), held("key_2", "key_2:secret"));
        write_map(&mock, &map);
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };
        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("key_1:secret")
        );

        map.active_workspace_id = Some("w2".to_string());
        write_map(&mock, &map);

        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("key_2:secret")
        );
    }

    // An org-less mint is bound to no workspace, so `keys` has nowhere to
    // put it. It must still be served.
    #[test]
    fn an_unbound_key_is_served_when_no_workspace_is_active() {
        let mock = Arc::new(MockKeyringStore::new());
        write_map(
            &mock,
            &WorkspaceKeyMap {
                active_workspace_id: None,
                keys: BTreeMap::new(),
                unbound_key: Some(held("key_free", "key_free:secret")),
            },
        );
        let store = ActiveKeyStore { inner: mock };

        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("key_free:secret")
        );
    }

    // A workspace selection must not be shadowed by an unbound key left over
    // from an earlier personal mint.
    #[test]
    fn an_active_workspace_beats_a_leftover_unbound_key() {
        let mock = Arc::new(MockKeyringStore::new());
        let mut map = bound("w1", "key_1", "key_1:secret");
        map.unbound_key = Some(held("key_free", "key_free:secret"));
        write_map(&mock, &map);
        let store = ActiveKeyStore { inner: mock };

        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("key_1:secret")
        );
    }

    // The whole point of map-first: an item frozen at upgrade time must not
    // keep serving the workspace the user has since switched away from.
    #[test]
    fn a_stale_mirror_item_does_not_shadow_the_map() {
        let mock = Arc::new(MockKeyringStore::new());
        write_map(&mock, &bound("w2", "key_2", "key_2:secret"));
        mock.set(CLI, KEY_SCHEME, "key_1:stale").unwrap();
        let store = ActiveKeyStore { inner: mock };

        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("key_2:secret"),
            "the map is the maintained copy; the leftover item is not"
        );
    }

    // `auth login --with-token` on an install that never logged in.
    #[test]
    fn a_real_item_serves_when_the_map_has_no_answer() {
        let mock = Arc::new(MockKeyringStore::new());
        mock.set(CLI, KEY_SCHEME, "pasted:token").unwrap();
        let store = ActiveKeyStore { inner: mock };

        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("pasted:token")
        );
    }

    // An empty map is not an answer, so the fallback still applies.
    #[test]
    fn an_active_marker_with_no_matching_entry_falls_through() {
        let mock = Arc::new(MockKeyringStore::new());
        write_map(
            &mock,
            &WorkspaceKeyMap {
                active_workspace_id: Some("w9".to_string()),
                keys: BTreeMap::new(),
                unbound_key: None,
            },
        );
        mock.set(CLI, KEY_SCHEME, "pasted:token").unwrap();
        let store = ActiveKeyStore { inner: mock };

        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("pasted:token")
        );
    }

    #[test]
    fn nothing_anywhere_is_still_nothing() {
        let store = ActiveKeyStore {
            inner: Arc::new(MockKeyringStore::new()),
        };
        assert!(store.get(CLI, KEY_SCHEME).unwrap().is_none());
    }

    // A corrupt map degrades to "no credential" — which the auth path
    // reports well — rather than erroring from inside a credential lookup.
    #[test]
    fn an_unparseable_map_falls_through_instead_of_erroring() {
        let mock = Arc::new(MockKeyringStore::new());
        mock.set(CLI, WORKSPACE_KEYS_SCHEME, "{ not json").unwrap();
        mock.set(CLI, KEY_SCHEME, "pasted:token").unwrap();
        let store = ActiveKeyStore { inner: mock };

        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("pasted:token")
        );
    }

    // Only the KeyAuth address is projected; every other slot is ordinary
    // storage.
    #[test]
    fn other_accounts_pass_straight_through() {
        let mock = Arc::new(MockKeyringStore::new());
        write_map(&mock, &bound("w1", "key_1", "key_1:secret"));
        mock.set(CLI, "OAuth", "bundle").unwrap();
        let store = ActiveKeyStore { inner: mock };

        assert_eq!(store.get(CLI, "OAuth").unwrap().as_deref(), Some("bundle"));
        assert!(store.get(CLI, "OAuthDiscovery").unwrap().is_none());
    }

    #[test]
    fn writes_and_deletes_pass_through() {
        let mock = Arc::new(MockKeyringStore::new());
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };

        store.set(CLI, KEY_SCHEME, "pasted:token").unwrap();
        assert_eq!(
            mock.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("pasted:token"),
            "a token paste must land in the backend, not be swallowed"
        );

        store.delete(CLI, KEY_SCHEME).unwrap();
        assert!(
            mock.get(CLI, KEY_SCHEME).unwrap().is_none(),
            "logout must actually remove the item"
        );
    }

    #[test]
    fn backend_label_passes_through() {
        let store = ActiveKeyStore {
            inner: Arc::new(MockKeyringStore::new()),
        };
        assert_eq!(
            store.backend_label(),
            MockKeyringStore::new().backend_label()
        );
    }
}

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
//! * `auth login --with-token` (`run_token_paste`) writes through this store
//!   like every other writer, so on an install that has a map the paste
//!   lands *in* the map and there is no second intent to reconcile. Only an
//!   install with no map at all writes a raw item, and there the fallback is
//!   the only answer anyway. `HEDRA_API_KEY` still shadows both.
//!
//! ## Symmetry
//!
//! `get`, `set` and `delete` all resolve the `KeyAuth` slot the same way.
//! That is the whole contract, and it is not decoration: while only `get`
//! was projected, `auth logout` deleted a raw item that was not the
//! credential in use and reported success with the CLI still authenticated,
//! and `auth login --with-token` wrote a raw item the projection then
//! ignored. A store that answers reads from one place and writes to another
//! will always produce a command that lies about what it did.
//!
//! [`ActiveKeyStore::write_map`] deletes the leftover item on every map
//! write — i.e. at exactly the moments the active credential changes — so an
//! upgraded install converges on one item rather than carrying an
//! unmaintained credential forever.
//!
//! ## Rollback
//!
//! A binary older than this projection reads only the standalone `KeyAuth`
//! item and cannot see the map, so **downgrading to v2.0.2 or earlier reads
//! as logged out**. That is intended, not an oversight: one `auth login`
//! restores it, and the alternative — keeping a mirrored copy in the old
//! address forever — costs a second keychain item and its own macOS
//! authorization prompt on every `workspaces select`, permanently, to spare
//! a downgrade one command. The choice was cheap to make because the
//! map-only store had not shipped in any release when it was taken
//! (ENG-10414; v2.0.2 was current and predates it).
//!
//! Hand-written and .fernignore-protected — the generator never emits this
//! file; the ignore entry is what stops regeneration from deleting it.

use std::sync::Arc;

use fern_cli_sdk::auth::KeyringStore;
use fern_cli_sdk::error::CliError;

use super::auth::KEY_SCHEME;
use super::workspaces::{HeldKey, WorkspaceKeyMap, WORKSPACE_KEYS_SCHEME};

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
        Ok(self
            .map(service)?
            .and_then(|map| map.active_credential().map(str::to_string)))
    }

    /// The stored map, or `None` when there is no map to speak of.
    ///
    /// A map that will not parse reads as absent, for the same reason
    /// [`Self::active_credential`] treats it that way: every operation then
    /// degrades to the raw-item path, which is a state the auth surface
    /// already describes well. The alternative — overwriting a map we cannot
    /// read — would destroy credentials to fix a display problem.
    fn map(&self, service: &str) -> Result<Option<WorkspaceKeyMap>, CliError> {
        let Some(raw) = self.inner.get(service, WORKSPACE_KEYS_SCHEME)? else {
            return Ok(None);
        };
        Ok(serde_json::from_str::<WorkspaceKeyMap>(&raw).ok())
    }

    fn save(&self, service: &str, map: &WorkspaceKeyMap) -> Result<(), CliError> {
        let json = serde_json::to_string(map)
            .map_err(|e| CliError::Auth(format!("could not serialize workspace key map: {e}")))?;
        self.write_map(service, &json)
    }

    /// Persist a serialized map, shedding the legacy standalone item once
    /// the map can answer for the active credential.
    ///
    /// Releases before the projection stored the active credential twice:
    /// inside the map, and again as its own `(cli_name, KeyAuth)` item. Once
    /// the map holds an active credential that item is dead weight — inert,
    /// because `get` prefers the map, but still a live credential sitting in
    /// the keychain that nothing updates, and on macOS still its own
    /// authorization prompt.
    ///
    /// The condition is load-bearing, not a tidy-up: a map write that files
    /// a key *without* activating it leaves `get` still falling back to the
    /// item, so dropping it there would delete the credential the user is
    /// actually presenting. The compatibility guard in `mint_for_workspace`
    /// does exactly that — it records a key that landed on the wrong
    /// workspace and deliberately leaves the active credential alone — and
    /// an unconditional shed turned that careful non-move into a logout.
    ///
    /// Doing it here rather than from each caller is what makes it correct:
    /// the previous arrangement had callers invoke a helper that deleted the
    /// `KeyAuth` slot through this very store, which — now that deleting
    /// that slot means "log out" — would take the map with it.
    ///
    /// Best-effort: a fresh install has nothing to delete, and failing to
    /// shed a redundant copy must not sink the write that matters.
    fn write_map(&self, service: &str, json: &str) -> Result<(), CliError> {
        self.inner.set(service, WORKSPACE_KEYS_SCHEME, json)?;
        let map_is_authoritative = serde_json::from_str::<WorkspaceKeyMap>(json)
            .is_ok_and(|map| map.active_credential().is_some());
        if map_is_authoritative {
            let _ = self.inner.delete(service, KEY_SCHEME);
        }
        Ok(())
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

    /// A write to the `KeyAuth` slot lands wherever `get` will look for it.
    ///
    /// The SDK's token-paste path is the only writer left, and it means
    /// "make this the credential I present". Passing it through to a raw
    /// item made that a lie on any install that had a map: the paste
    /// succeeded, said so, and `get` went on serving the map's older answer
    /// — so a user rotating a leaked key kept sending the leaked one.
    ///
    /// A pasted credential goes to `unbound_key`, not over the active
    /// workspace's entry. The map is indexed by workspace and the CLI has no
    /// idea which workspace a pasted key belongs to; filing it under the
    /// workspace that happened to be active would attribute it to a
    /// workspace it may have nothing to do with. `unbound_key` is precisely
    /// the "active credential bound to no workspace" slot, and clearing the
    /// marker alongside it is what `record_key` already does for an
    /// unbound mint.
    fn set(&self, service: &str, account: &str, value: &str) -> Result<(), CliError> {
        // A direct map write — `WorkspaceKeyMap::save`, i.e. every mint,
        // renewal and `workspaces select`. Routed through `write_map` so it
        // sheds the legacy mirror like any other map write.
        if account == WORKSPACE_KEYS_SCHEME {
            return self.write_map(service, value);
        }
        if account != KEY_SCHEME {
            return self.inner.set(service, account, value);
        }
        let Some(mut map) = self.map(service)? else {
            // No map: the raw item is what `get` falls back to, so it is
            // also the right place to write. A user who has never logged in
            // does not acquire a workspace map by pasting a key.
            return self.inner.set(service, account, value);
        };
        map.unbound_key = Some(HeldKey {
            // Credentials are `<key_id>:<secret>`. Without a separator we
            // have no id — and must not fall back to the whole value, which
            // would print the secret wherever the key id is displayed.
            key_id: value
                .split_once(':')
                .map(|(id, _)| id.to_string())
                .unwrap_or_else(|| "(pasted)".to_string()),
            credential: value.to_string(),
            workspace_name: None,
            expires_at: None,
        });
        map.active_workspace_id = None;
        self.save(service, &map)
    }

    /// Logging out of `KeyAuth` clears **every** credential this slot can
    /// serve: the whole workspace map, and the legacy standalone item.
    ///
    /// Deleting only the raw item — the old behaviour — let `auth logout`
    /// exit zero, print that the credential was removed, and leave the CLI
    /// fully authenticated from the map. That is the regression this store
    /// exists to prevent, so the delete has to reach the same places the
    /// read does.
    ///
    /// The whole map goes, not just the active entry. Every key in it is a
    /// live credential the CLI can present on demand: leaving the others
    /// behind would mean `workspaces select <other>` re-authenticates with
    /// no challenge at all, which makes "logged out" false for anyone who
    /// has ever held a second workspace. Re-minting after a login is cheap;
    /// a logout that leaves usable secrets on disk is not.
    ///
    /// Both deletes are attempted even if the first fails, so one wedged
    /// backend cannot strand the other copy; the first error is reported.
    fn delete(&self, service: &str, account: &str) -> Result<(), CliError> {
        if account != KEY_SCHEME {
            return self.inner.delete(service, account);
        }
        let map = self.inner.delete(service, WORKSPACE_KEYS_SCHEME);
        let item = self.inner.delete(service, KEY_SCHEME);
        map.and(item)
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

    // With no map, a write is an ordinary item write — that item is what
    // `get` falls back to, so it is also where the paste belongs.
    #[test]
    fn a_paste_writes_a_real_item_when_there_is_no_map() {
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
        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("pasted:token"),
            "and it must be what resolves afterwards"
        );
    }

    // The regression: a paste on a migrated install used to write a raw item
    // that `get` then ignored in favour of the map, so a rotated key never
    // took effect. The assertion that matters is the resolved credential.
    #[test]
    fn a_paste_replaces_the_credential_the_map_serves() {
        let mock = Arc::new(MockKeyringStore::new());
        write_map(&mock, &bound("w1", "key_1", "key_1:old"));
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };

        store.set(CLI, KEY_SCHEME, "key_9:rotated").unwrap();

        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("key_9:rotated"),
            "the pasted key must be the credential in use, not the map's older answer"
        );
    }

    // A pasted key belongs to no known workspace, so it must not be filed
    // under whichever one happened to be active.
    #[test]
    fn a_pasted_key_is_unbound_and_leaves_held_keys_alone() {
        let mock = Arc::new(MockKeyringStore::new());
        write_map(&mock, &bound("w1", "key_1", "key_1:old"));
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };

        store.set(CLI, KEY_SCHEME, "key_9:rotated").unwrap();

        let raw = mock.get(CLI, WORKSPACE_KEYS_SCHEME).unwrap().unwrap();
        let map: WorkspaceKeyMap = serde_json::from_str(&raw).unwrap();
        assert_eq!(map.active_workspace_id, None, "the marker clears");
        assert_eq!(
            map.unbound_key.as_ref().map(|k| k.credential.as_str()),
            Some("key_9:rotated")
        );
        assert_eq!(
            map.keys.get("w1").map(|k| k.credential.as_str()),
            Some("key_1:old"),
            "w1's own key is untouched — the paste said nothing about it"
        );
    }

    // A credential with no `<key_id>:` prefix must never have the secret
    // itself recorded as the key id, which is displayed.
    #[test]
    fn a_separatorless_paste_does_not_become_its_own_key_id() {
        let mock = Arc::new(MockKeyringStore::new());
        write_map(&mock, &bound("w1", "key_1", "key_1:old"));
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };

        store.set(CLI, KEY_SCHEME, "opaque-secret").unwrap();

        let raw = mock.get(CLI, WORKSPACE_KEYS_SCHEME).unwrap().unwrap();
        let map: WorkspaceKeyMap = serde_json::from_str(&raw).unwrap();
        let held = map.unbound_key.as_ref().unwrap();
        assert_eq!(held.credential, "opaque-secret");
        assert_ne!(
            held.key_id, "opaque-secret",
            "the key id is displayed; it must not be the secret"
        );
    }

    // The headline regression, at the level that matters: after a logout the
    // slot must resolve to nothing. Deleting the raw item alone left the map
    // answering, so `auth status` stayed active and requests stayed signed.
    #[test]
    fn logout_leaves_no_resolvable_credential() {
        let mock = Arc::new(MockKeyringStore::new());
        write_map(&mock, &bound("w1", "key_1", "key_1:secret"));
        mock.set(CLI, KEY_SCHEME, "key_0:legacy").unwrap();
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };

        store.delete(CLI, KEY_SCHEME).unwrap();

        assert!(
            store.get(CLI, KEY_SCHEME).unwrap().is_none(),
            "logout must leave the projected credential unresolvable"
        );
        assert!(
            mock.get(CLI, KEY_SCHEME).unwrap().is_none(),
            "the legacy mirror goes too"
        );
        assert!(
            mock.get(CLI, WORKSPACE_KEYS_SCHEME).unwrap().is_none(),
            "and so does the map that was actually serving it"
        );
    }

    // Every held key is a live credential; leaving the non-active ones would
    // let `workspaces select <other>` re-authenticate with no challenge.
    #[test]
    fn logout_clears_keys_held_for_other_workspaces_too() {
        let mock = Arc::new(MockKeyringStore::new());
        let mut map = bound("w1", "key_1", "key_1:secret");
        let _ = map
            .keys
            .insert("w2".to_string(), held("key_2", "key_2:secret"));
        write_map(&mock, &map);
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };

        store.delete(CLI, KEY_SCHEME).unwrap();

        assert!(
            mock.get(CLI, WORKSPACE_KEYS_SCHEME).unwrap().is_none(),
            "a logout that leaves usable secrets on disk is not a logout"
        );
    }

    // Filing a key without activating it must leave the legacy item alone:
    // it is still what `get` falls back to, so shedding it there would
    // delete the credential the user is currently presenting. This is the
    // shape `mint_for_workspace`'s compatibility guard produces.
    #[test]
    fn filing_an_inactive_key_keeps_the_legacy_item() {
        let mock = Arc::new(MockKeyringStore::new());
        mock.set(CLI, KEY_SCHEME, "key_held:stay").unwrap();
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };

        // A map that holds a key but activates nothing.
        let mut map = WorkspaceKeyMap::default();
        let _ = map
            .keys
            .insert("w1".to_string(), held("key_1", "key_1:elsewhere"));
        store
            .set(
                CLI,
                WORKSPACE_KEYS_SCHEME,
                &serde_json::to_string(&map).unwrap(),
            )
            .unwrap();

        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("key_held:stay"),
            "the active credential must survive filing a key for another workspace"
        );
    }

    // Once the map does hold an active credential, the redundant item goes.
    #[test]
    fn activating_through_the_map_sheds_the_legacy_item() {
        let mock = Arc::new(MockKeyringStore::new());
        mock.set(CLI, KEY_SCHEME, "key_held:stale").unwrap();
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };

        store
            .set(
                CLI,
                WORKSPACE_KEYS_SCHEME,
                &serde_json::to_string(&bound("w1", "key_1", "key_1:secret")).unwrap(),
            )
            .unwrap();

        assert!(
            mock.get(CLI, KEY_SCHEME).unwrap().is_none(),
            "the unmaintained copy must not linger once the map answers"
        );
        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("key_1:secret")
        );
    }

    // Logging out of an install that never migrated must still work.
    #[test]
    fn logout_works_with_only_a_legacy_item() {
        let mock = Arc::new(MockKeyringStore::new());
        mock.set(CLI, KEY_SCHEME, "key_0:legacy").unwrap();
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };

        store.delete(CLI, KEY_SCHEME).unwrap();

        assert!(store.get(CLI, KEY_SCHEME).unwrap().is_none());
    }

    // Deleting another scheme must not touch the key map.
    #[test]
    fn deleting_a_different_scheme_leaves_the_map_alone() {
        let mock = Arc::new(MockKeyringStore::new());
        write_map(&mock, &bound("w1", "key_1", "key_1:secret"));
        mock.set(CLI, "OAuth", "bundle").unwrap();
        let store = ActiveKeyStore {
            inner: mock.clone(),
        };

        store.delete(CLI, "OAuth").unwrap();

        assert!(mock.get(CLI, "OAuth").unwrap().is_none());
        assert_eq!(
            store.get(CLI, KEY_SCHEME).unwrap().as_deref(),
            Some("key_1:secret"),
            "the data-plane credential survives an OAuth logout"
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

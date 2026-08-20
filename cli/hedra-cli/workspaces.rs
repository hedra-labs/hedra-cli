//! Workspace surface: the v3 login-plane workspace listing (ENG-10346's
//! picker source), the local per-workspace key map, and the `workspaces`
//! command (`list` / `select`).
//!
//! The CLI can only ever HOLD keys it got from logins — the bootstrap mint
//! takes no workspace parameter (the JWT's org is the sole selector) and a
//! bootstrapped key deliberately lacks KEYS_MANAGE, so it cannot mint keys
//! for other workspaces. Keys therefore accumulate here, one per workspace
//! logged into, and `select` switches the active `KeyAuth` slot between
//! them — auto-launching a login (with a WorkOS organization hint) when no
//! key is held for the target.
//!
//! Hand-written and .fernignore-protected — the generator never emits this
//! file; the ignore entry is what stops regeneration from deleting it.
//! Declared from `custom.rs` via `#[path]` so the generated `main.rs`
//! stays untouched.

use std::collections::BTreeMap;

use fern_cli_sdk::auth::{active_store, LoginContext, LoginFlow};
use fern_cli_sdk::error::CliError;
use fern_cli_sdk::openapi::AppContext;

use super::auth;

/// One row of `GET /v3/workspaces` — mirrors the workspace summary the
/// login plane returns.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct WorkspaceSummary {
    workspace_id: String,
    workspace_name: String,
    role: String,
    #[serde(default)]
    workos_organization_id: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct WorkspaceListResponse {
    data: Vec<WorkspaceSummary>,
}

// ---------------------------------------------------------------------------
// Local key map: one bootstrapped key per workspace, in a single keyring slot.
// ---------------------------------------------------------------------------

/// Keyring slot holding the JSON [`WorkspaceKeyMap`]. This is the *only*
/// slot the active credential lives in: the `KeyAuth` address the SDK's
/// keyring source reads is projected from this map at resolve time by
/// [`super::active_key`], rather than being a second item holding a copy.
pub(crate) const WORKSPACE_KEYS_SCHEME: &str = "WorkspaceKeys";

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct HeldKey {
    pub(crate) key_id: String,
    /// The full `<key_id>:<secret>` pair, as stored in `KeyAuth` when active.
    pub(crate) credential: String,
    #[serde(default)]
    pub(crate) workspace_name: Option<String>,
    #[serde(default)]
    pub(crate) expires_at: Option<String>,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub(crate) struct WorkspaceKeyMap {
    /// Workspace of the currently active credential; `None` when the active
    /// key is unbound (org-less personal mint), in which case it lives in
    /// [`unbound_key`](Self::unbound_key) instead.
    #[serde(default)]
    pub(crate) active_workspace_id: Option<String>,
    #[serde(default)]
    pub(crate) keys: BTreeMap<String, HeldKey>,
    /// The active credential when it is bound to no workspace. `keys` is
    /// indexed by workspace id and so has nowhere to put one; before the
    /// `KeyAuth` slot became a projection of this map, an org-less mint
    /// survived only as that separate item. Without this field it would
    /// have nowhere to live at all.
    #[serde(default)]
    pub(crate) unbound_key: Option<HeldKey>,
}

impl WorkspaceKeyMap {
    pub(crate) fn load(cli_name: &str) -> Self {
        match active_store().get(cli_name, WORKSPACE_KEYS_SCHEME) {
            Ok(Some(raw)) => serde_json::from_str(&raw).unwrap_or_default(),
            _ => Self::default(),
        }
    }

    /// The credential the CLI should present right now, if it holds one.
    ///
    /// `active_workspace_id` wins: selecting a workspace is what makes its
    /// key active, and an `unbound_key` left over from an earlier org-less
    /// mint must not shadow that choice.
    pub(crate) fn active_credential(&self) -> Option<&str> {
        match &self.active_workspace_id {
            Some(id) => self.keys.get(id).map(|k| k.credential.as_str()),
            None => self.unbound_key.as_ref().map(|k| k.credential.as_str()),
        }
    }

    fn save(&self, cli_name: &str) -> Result<(), CliError> {
        let json = serde_json::to_string(self)
            .map_err(|e| CliError::Auth(format!("could not serialize workspace key map: {e}")))?;
        active_store().set(cli_name, WORKSPACE_KEYS_SCHEME, &json)
    }
}

/// Record a key the bootstrap just minted or renewed, and mark it active
/// (it IS the credential the `KeyAuth` slot now projects).
/// A `None` workspace means an unbound personal key: it is filed in
/// `unbound_key` rather than in `keys`, and the workspace marker clears so
/// no stale star is shown.
/// A `None` name preserves any name already held (renewals don't carry one).
/// `activate: false` files the key WITHOUT making it the active credential — the ENG-10403 compatibility guard uses it to keep a
/// key that landed on the wrong workspace instead of orphaning it, while
/// leaving the user's active workspace untouched.
#[allow(clippy::too_many_arguments)]
pub(crate) fn record_key(
    cli_name: &str,
    workspace_id: Option<&str>,
    key_id: &str,
    credential: &str,
    workspace_name: Option<&str>,
    expires_at: Option<&str>,
    activate: bool,
) -> Result<(), CliError> {
    let mut map = WorkspaceKeyMap::load(cli_name);
    match workspace_id {
        Some(ws) => {
            let kept_name = map.keys.get(ws).and_then(|k| k.workspace_name.clone());
            map.keys.insert(
                ws.to_string(),
                HeldKey {
                    key_id: key_id.to_string(),
                    credential: credential.to_string(),
                    workspace_name: workspace_name.map(str::to_string).or(kept_name),
                    expires_at: expires_at.map(str::to_string),
                },
            );
            if activate {
                map.active_workspace_id = Some(ws.to_string());
            }
        }
        // An unbound mint owns no *workspace* entry — `keys` is indexed by
        // workspace id. It still has to be stored, because the map is the
        // only place the active credential lives, so it goes in its own
        // slot and the workspace marker clears (no stale star).
        None if activate => {
            map.unbound_key = Some(HeldKey {
                key_id: key_id.to_string(),
                credential: credential.to_string(),
                workspace_name: workspace_name.map(str::to_string),
                expires_at: expires_at.map(str::to_string),
            });
            map.active_workspace_id = None;
        }
        None => {}
    }
    map.save(cli_name)
}

pub(crate) enum SelectOutcome {
    Activated(HeldKey),
    NotHeld,
}

/// Make the held key for `workspace_id` the active credential. Local-only:
/// moves the active marker, which is what the `KeyAuth` projection reads.
pub(crate) fn activate(cli_name: &str, workspace_id: &str) -> Result<SelectOutcome, CliError> {
    let mut map = WorkspaceKeyMap::load(cli_name);
    let Some(key) = map.keys.get(workspace_id).cloned() else {
        return Ok(SelectOutcome::NotHeld);
    };
    map.active_workspace_id = Some(workspace_id.to_string());
    map.save(cli_name)?;
    auth::drop_stale_key_mirror(cli_name);
    Ok(SelectOutcome::Activated(key))
}

// ---------------------------------------------------------------------------
// Listing fetch + render.
// ---------------------------------------------------------------------------

/// Send a fully-built (and already authenticated) `GET {base}/v3/workspaces`
/// request and parse the listing. Auth is applied by the caller: the
/// bootstrap passes the login JWT, the `workspaces` command routes through
/// the OAuth provider (which refreshes an expired token).
async fn fetch_workspaces_request(
    req: reqwest::RequestBuilder,
) -> Result<Vec<WorkspaceSummary>, CliError> {
    let resp = req
        .send()
        .await
        .map_err(|e| CliError::Auth(format!("GET /v3/workspaces failed: {e}")))?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(CliError::Auth(format!(
            "workspace listing refused — {}",
            auth::login_plane_error(status, &body)
        )));
    }
    let listing: WorkspaceListResponse = serde_json::from_str(&body)
        .map_err(|e| CliError::Auth(format!("unexpected /v3/workspaces response: {e}")))?;
    Ok(listing.data)
}

/// Fetch the picker listing with the login JWT (the bootstrap path). The
/// JWT is deliberately not consumed by the server, so list-then-mint works
/// on one login.
pub(crate) async fn fetch_workspaces(
    http: &reqwest::Client,
    api_base: &str,
    jwt: &str,
) -> Result<Vec<WorkspaceSummary>, CliError> {
    fetch_workspaces_request(
        http.get(format!("{api_base}/v3/workspaces"))
            .bearer_auth(jwt),
    )
    .await
}

/// Render the listing: `★` on the active workspace, name + id, and a
/// `key held` marker on workspaces `select` can switch to offline.
pub(crate) fn render_workspace_table(
    workspaces: &[WorkspaceSummary],
    active_id: Option<&str>,
    held: &BTreeMap<String, HeldKey>,
) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    if workspaces.is_empty() {
        out.push_str("No workspaces visible to this account.\n");
        return out;
    }
    let name_w = workspaces
        .iter()
        .map(|w| w.workspace_name.chars().count())
        .max()
        .unwrap_or(0);
    let id_w = workspaces
        .iter()
        .map(|w| w.workspace_id.chars().count())
        .max()
        .unwrap_or(0);
    let _ = writeln!(out, "Workspaces ({}):", workspaces.len());
    for ws in workspaces {
        let star = if active_id == Some(ws.workspace_id.as_str()) {
            '★'
        } else {
            ' '
        };
        let org = ws
            .workos_organization_id
            .as_deref()
            .map(|o| format!("org: {o}"))
            .unwrap_or_else(|| "personal".to_string());
        let held_marker = if held.contains_key(&ws.workspace_id) {
            "  [key held]"
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "  {star} {:name_w$}  {:id_w$}  role: {:<7} {org}{held_marker}",
            ws.workspace_name, ws.workspace_id, ws.role
        );
    }
    match active_id {
        Some(id) if !workspaces.iter().any(|w| w.workspace_id == id) => {
            let _ = writeln!(
                out,
                "  ★ active key is bound to workspace {id} (not in this listing)"
            );
        }
        None => {
            let _ = writeln!(out, "  (no workspace-bound key is active)");
        }
        _ => {}
    }
    out
}

// ---------------------------------------------------------------------------
// The `workspaces` command.
// ---------------------------------------------------------------------------

pub(crate) fn command() -> clap::Command {
    clap::Command::new("workspaces")
        .about("List workspaces and select the active one")
        .long_about(
            "List the workspaces this account can access and switch which \
             workspace's API key is the active credential. Bare `workspaces` \
             is `workspaces list`.",
        )
        .subcommand(
            clap::Command::new("list").about("List accessible workspaces (the default subcommand)"),
        )
        .subcommand(
            clap::Command::new("select")
                .about("Make the API key bound to a workspace the active credential")
                .long_about(
                    "Switch the active credential to the held key for the given \
                     workspace. When no key is held for it, one is minted for that \
                     workspace from the current login — no browser round-trip \
                     unless you are not logged in at all. Works for workspaces with \
                     no linked organization, which no login can select by itself.",
                )
                .arg(
                    clap::Arg::new("workspace-id")
                        .long("workspace-id")
                        .alias("workspace_id")
                        .value_name("WORKSPACE_ID")
                        .required(true)
                        .help("Target workspace id (see `workspaces list`)"),
                ),
        )
}

/// Root handler for the `workspaces` group — sub-dispatches on the parsed
/// subcommand; bare `workspaces` defaults to `list`.
pub(crate) fn dispatch(
    matches: &clap::ArgMatches,
    ctx: &dyn std::any::Any,
) -> Result<(), CliError> {
    let cli_name = ctx
        .downcast_ref::<AppContext>()
        .map(|c| c.http_config().name().to_string())
        .ok_or_else(|| {
            CliError::Validation("workspaces: unexpected binding context type".to_string())
        })?;
    match matches.subcommand() {
        None | Some(("list", _)) => run_list(&cli_name),
        Some(("select", sub)) => {
            let ws = sub
                .get_one::<String>("workspace-id")
                .expect("--workspace-id is required");
            run_select(&cli_name, ws)
        }
        Some((other, _)) => Err(CliError::Validation(format!(
            "unknown workspaces subcommand: {other}"
        ))),
    }
}

/// Run a future on the app's runtime from this sync handler — the same
/// pattern the bootstrap uses.
fn run_async<T>(fut: impl std::future::Future<Output = T>) -> T {
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
}

fn http_client() -> Result<reqwest::Client, CliError> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| CliError::Auth(format!("could not build HTTP client: {e}")))
}

/// Fetch the listing as the logged-in user. `/v3/workspaces` is a
/// login-plane endpoint, so it needs a freshly minted JWT rather than
/// whatever unexpired token the keyring happens to hold — see
/// `auth::fresh_login_jwt`.
fn fetch_listing_as_user(cli_name: &str) -> Result<Vec<WorkspaceSummary>, CliError> {
    let http = http_client()?;
    let jwt = auth::fresh_login_jwt(cli_name)?;
    let url = format!("{}/v3/workspaces", auth::resource_base_url());
    run_async(fetch_workspaces_request(http.get(url).bearer_auth(jwt)))
}

fn warn_if_env_key_shadows() {
    if std::env::var("HEDRA_API_KEY").is_ok() {
        eprintln!("⚠ HEDRA_API_KEY is set and shadows the keyring — the active workspace key is not what requests will use.");
    }
}

fn run_list(cli_name: &str) -> Result<(), CliError> {
    let listing = fetch_listing_as_user(cli_name)?;
    let map = WorkspaceKeyMap::load(cli_name);
    print!(
        "{}",
        render_workspace_table(&listing, map.active_workspace_id.as_deref(), &map.keys)
    );
    warn_if_env_key_shadows();
    Ok(())
}

fn run_select(cli_name: &str, workspace_id: &str) -> Result<(), CliError> {
    if let SelectOutcome::Activated(key) = activate(cli_name, workspace_id)? {
        announce_active(
            workspace_id,
            key.workspace_name.as_deref(),
            Some(&key.key_id),
        );
        return Ok(());
    }

    // No key held for the target. A browser login is needed only when there
    // is no session at all — the workspace itself is named at mint time, so
    // an existing session (refreshed) can mint for any workspace the account
    // is a member of, whether or not it has a WorkOS organization.
    if !auth::has_oauth_session(cli_name) {
        eprintln!("Not logged in — launching browser login…");
        auth::EnvPkceLoginFlow::new().run(&LoginContext {
            cli_name: cli_name.to_string(),
            no_browser: false,
        })?;
        // The login's own bootstrap may already have landed on the target.
        if let SelectOutcome::Activated(key) = activate(cli_name, workspace_id)? {
            announce_active(
                workspace_id,
                key.workspace_name.as_deref(),
                Some(&key.key_id),
            );
            return Ok(());
        }
    }

    // Fail on a typo'd or invisible id before minting anything; the listing
    // also supplies the display name for the confirmation line.
    let listing = fetch_listing_as_user(cli_name)?;
    let Some(target) = listing.iter().find(|w| w.workspace_id == workspace_id) else {
        return Err(CliError::Validation(format!(
            "workspace {workspace_id} is not visible to this account; known ids: {}",
            listing
                .iter()
                .map(|w| w.workspace_id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )));
    };

    eprintln!(
        "No API key held for {} ({workspace_id}) — minting one…",
        target.workspace_name
    );
    let minted = auth::mint_for_workspace(cli_name, workspace_id)?;
    announce_active(
        workspace_id,
        minted
            .workspace_name
            .as_deref()
            .or(Some(target.workspace_name.as_str())),
        Some(&minted.key_id),
    );
    if let Some(expiry) = minted.expires_at.as_deref() {
        eprintln!("(key expires {expiry})");
    }
    Ok(())
}

fn announce_active(workspace_id: &str, name: Option<&str>, key_id: Option<&str>) {
    let name = name.unwrap_or("<unnamed>");
    match key_id {
        Some(id) => println!("Active workspace: {name} ({workspace_id}) — key {id}"),
        None => println!("Active workspace: {name} ({workspace_id})"),
    }
    warn_if_env_key_shadows();
}

#[cfg(test)]
mod tests {
    use super::*;
    use fern_cli_sdk::auth::{KeyringStore, MockKeyringStore};

    fn ws(id: &str, name: &str, org: Option<&str>) -> WorkspaceSummary {
        WorkspaceSummary {
            workspace_id: id.to_string(),
            workspace_name: name.to_string(),
            role: "member".to_string(),
            workos_organization_id: org.map(str::to_string),
        }
    }

    fn held(key_id: &str) -> HeldKey {
        HeldKey {
            key_id: key_id.to_string(),
            credential: format!("{key_id}:secret"),
            workspace_name: None,
            expires_at: None,
        }
    }

    /// Installs the `KeyAuth` projection over the mock, as production does,
    /// so assertions about the active credential go through the derivation.
    fn fresh_keyring() -> std::sync::Arc<MockKeyringStore> {
        super::super::active_key::projected_mock()
    }

    // ── key map ─────────────────────────────────────────────────────────

    #[test]
    #[serial_test::serial]
    fn key_map_defaults_on_absent_or_garbage() {
        let store = fresh_keyring();
        assert!(WorkspaceKeyMap::load("test-cli").keys.is_empty());
        store
            .set("test-cli", WORKSPACE_KEYS_SCHEME, "not json")
            .unwrap();
        let map = WorkspaceKeyMap::load("test-cli");
        assert!(map.keys.is_empty());
        assert_eq!(map.active_workspace_id, None);
    }

    #[test]
    #[serial_test::serial]
    fn record_key_upserts_and_marks_active() {
        fresh_keyring();
        record_key(
            "test-cli",
            Some("w1"),
            "key_1",
            "key_1:s3cret",
            Some("Acme"),
            Some("2026-08-20T00:00:00Z"),
            true,
        )
        .unwrap();
        let map = WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id.as_deref(), Some("w1"));
        assert_eq!(map.keys["w1"].credential, "key_1:s3cret");
        assert_eq!(map.keys["w1"].workspace_name.as_deref(), Some("Acme"));

        // A renewal carries no name — the held one is preserved.
        record_key(
            "test-cli",
            Some("w1"),
            "key_1",
            "key_1:s3cret",
            None,
            None,
            true,
        )
        .unwrap();
        assert_eq!(
            WorkspaceKeyMap::load("test-cli").keys["w1"]
                .workspace_name
                .as_deref(),
            Some("Acme")
        );

        // An unbound mint clears the active marker but keeps held keys.
        record_key("test-cli", None, "key_2", "key_2:x", None, None, true).unwrap();
        let map = WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id, None);
        assert!(map.keys.contains_key("w1"));
    }

    #[test]
    #[serial_test::serial]
    fn activate_switches_the_key_auth_slot() {
        let store = fresh_keyring();
        record_key(
            "test-cli",
            Some("w1"),
            "key_1",
            "key_1:a",
            Some("A"),
            None,
            true,
        )
        .unwrap();
        record_key(
            "test-cli",
            Some("w2"),
            "key_2",
            "key_2:b",
            Some("B"),
            None,
            true,
        )
        .unwrap();
        store.set("test-cli", auth::KEY_SCHEME, "key_2:b").unwrap();

        let outcome = activate("test-cli", "w1").unwrap();
        assert!(matches!(outcome, SelectOutcome::Activated(ref k) if k.key_id == "key_1"));
        assert_eq!(
            active_store()
                .get("test-cli", auth::KEY_SCHEME)
                .unwrap()
                .as_deref(),
            Some("key_1:a")
        );
        assert_eq!(
            WorkspaceKeyMap::load("test-cli")
                .active_workspace_id
                .as_deref(),
            Some("w1")
        );
    }

    #[test]
    #[serial_test::serial]
    fn activate_unknown_workspace_is_not_held() {
        let store = fresh_keyring();
        record_key("test-cli", Some("w1"), "key_1", "key_1:a", None, None, true).unwrap();
        store.set("test-cli", auth::KEY_SCHEME, "key_1:a").unwrap();

        assert!(matches!(
            activate("test-cli", "w9").unwrap(),
            SelectOutcome::NotHeld
        ));
        // Nothing moved.
        assert_eq!(
            active_store()
                .get("test-cli", auth::KEY_SCHEME)
                .unwrap()
                .as_deref(),
            Some("key_1:a")
        );
        assert_eq!(
            WorkspaceKeyMap::load("test-cli")
                .active_workspace_id
                .as_deref(),
            Some("w1")
        );
    }

    // An org-less mint is bound to no workspace, so `keys` — indexed by
    // workspace id — has nowhere to put it. Before the KeyAuth slot became a
    // projection of this map it survived only as that separate item, so
    // without `unbound_key` the credential would simply be lost.
    #[test]
    #[serial_test::serial]
    fn an_unbound_mint_is_still_the_active_credential() {
        let _store = fresh_keyring();

        record_key("test-cli", None, "key_free", "key_free:x", None, None, true).unwrap();

        let map = WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_credential(), Some("key_free:x"));
        assert!(
            map.active_workspace_id.is_none(),
            "an unbound key must not leave a stale star on any workspace"
        );
        assert_eq!(
            active_store()
                .get("test-cli", auth::KEY_SCHEME)
                .unwrap()
                .as_deref(),
            Some("key_free:x"),
            "and it must reach the wire through the KeyAuth projection"
        );
    }

    // Upgrades: releases before the projection wrote a standalone KeyAuth
    // item. The map already beats it, but leaving it in place keeps an
    // unmaintained credential in the keychain, costing a prompt for nothing.
    #[test]
    #[serial_test::serial]
    fn activate_clears_a_stale_key_auth_mirror() {
        let store = fresh_keyring();
        record_key("test-cli", Some("w1"), "key_1", "key_1:a", None, None, true).unwrap();
        store
            .set("test-cli", auth::KEY_SCHEME, "key_9:stale")
            .unwrap();

        let _ = activate("test-cli", "w1").unwrap();

        assert!(
            store.get("test-cli", auth::KEY_SCHEME).unwrap().is_none(),
            "the legacy mirror item should be gone from the backend entirely"
        );
    }

    // ── render ──────────────────────────────────────────────────────────

    #[test]
    fn render_stars_active_and_marks_held() {
        let list = [
            ws("w1", "Personal", None),
            ws("w2", "Acme", Some("org_1")),
            ws("w3", "Big Corp", Some("org_9")),
        ];
        let mut held_keys = BTreeMap::new();
        held_keys.insert("w1".to_string(), held("key_1"));
        held_keys.insert("w2".to_string(), held("key_2"));

        let out = render_workspace_table(&list, Some("w2"), &held_keys);
        let acme = out.lines().find(|l| l.contains("Acme")).unwrap();
        assert!(acme.trim_start().starts_with('★'), "no star: {acme}");
        assert!(acme.contains("w2"), "id missing: {acme}");
        assert!(acme.contains("[key held]"));
        let personal = out.lines().find(|l| l.contains("Personal")).unwrap();
        assert!(!personal.contains('★'));
        assert!(personal.contains("[key held]"));
        let big = out.lines().find(|l| l.contains("Big Corp")).unwrap();
        assert!(!big.contains("[key held]"));
        assert!(big.contains("org: org_9"));
    }

    #[test]
    fn render_notes_unbound_and_unlisted_active() {
        let list = [ws("w1", "A", None), ws("w2", "B", Some("org_1"))];
        let none = render_workspace_table(&list, None, &BTreeMap::new());
        assert!(none.contains("no workspace-bound key is active"));
        let gone = render_workspace_table(&list, Some("w9"), &BTreeMap::new());
        assert!(gone.contains("bound to workspace w9"));
        assert!(
            render_workspace_table(&[], None, &BTreeMap::new()).contains("No workspaces visible")
        );
    }

    // ── command shape ───────────────────────────────────────────────────

    #[test]
    fn command_parses_bare_list_and_select() {
        let m = command().try_get_matches_from(["workspaces"]).unwrap();
        assert!(m.subcommand().is_none());

        let m = command()
            .try_get_matches_from(["workspaces", "select", "--workspace-id", "w1"])
            .unwrap();
        let ("select", sub) = m.subcommand().unwrap() else {
            panic!("expected select")
        };
        assert_eq!(sub.get_one::<String>("workspace-id").unwrap(), "w1");

        // The snake_case spelling works as an alias.
        let m = command()
            .try_get_matches_from(["workspaces", "select", "--workspace_id", "w2"])
            .unwrap();
        let ("select", sub) = m.subcommand().unwrap() else {
            panic!("expected select")
        };
        assert_eq!(sub.get_one::<String>("workspace-id").unwrap(), "w2");

        assert!(command()
            .try_get_matches_from(["workspaces", "select"])
            .is_err());
    }

    // ── listing fetch (wire) ────────────────────────────────────────────

    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn fetch_workspaces_sends_bearer_and_parses() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/workspaces"))
            .and(header("authorization", "Bearer test-jwt"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"workspace_id": "w1", "workspace_name": "P", "role": "owner",
                          "workos_organization_id": null}],
                "next_cursor": null,
            })))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let listing = fetch_workspaces(&http, &server.uri(), "test-jwt")
            .await
            .unwrap();
        assert_eq!(listing.len(), 1);
        assert_eq!(listing[0].workspace_id, "w1");
    }

    #[tokio::test]
    async fn fetch_workspaces_surfaces_login_plane_errors() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/workspaces"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "error": {"code": 401, "message": "token expired"},
            })))
            .mount(&server)
            .await;
        let http = reqwest::Client::new();
        let err = fetch_workspaces(&http, &server.uri(), "dead")
            .await
            .unwrap_err();
        assert!(err.to_string().contains("token expired"), "got: {err}");
    }
}

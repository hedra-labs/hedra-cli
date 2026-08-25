//! Workspace surface: the v3 login-plane workspace listing (the picker
//! source), the local per-workspace key map, and the `workspaces`
//! command (`list` / `select`).
//!
//! The CLI can only ever HOLD keys it got from logins — a bootstrapped key
//! deliberately lacks KEYS_MANAGE, so it cannot mint keys for other
//! workspaces. Keys therefore accumulate here, one per workspace logged
//! into, and `select` switches the active `KeyAuth` slot between them —
//! auto-launching a login when no key is held for the target.
//!
//! `select` renews the key it is about to activate rather than trusting it.
//! Holding a key is not the same as holding a working one, and activating
//! one blind made "the CLI says I am on this workspace" and "the CLI can
//! call this workspace" two different facts: a revoked key got a green
//! marker and then a 401 on every request. The renewal answers the question
//! and extends the key in one round-trip; a refusal means it is dead and a
//! replacement is minted.
//!
//! The mint itself does take a workspace (ENG-10403), which is what lets
//! `select` reach a workspace the login's organization does not name, and
//! what [`fallback_mint_target`] feeds when a login-time mint is refused
//! for exactly that reason.
//!
//! Hand-written and .fernignore-protected — the generator never emits this
//! file; the ignore entry is what stops regeneration from deleting it.
//! Declared from `custom.rs` via `#[path]` so the generated `main.rs`
//! stays untouched.

use std::collections::BTreeMap;

use anyhow::Context as _;
use fern_cli_sdk::auth::{active_store, LoginContext, LoginFlow};
use fern_cli_sdk::error::CliError;
use fern_cli_sdk::formatter::{format_value, OutputFormat, OutputPipeline};
use fern_cli_sdk::openapi::AppContext;
use serde_json::{json, Value};

use super::auth;

/// One row of `GET /v3/workspaces` — mirrors the workspace summary the
/// login plane returns.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct WorkspaceSummary {
    workspace_id: String,
    workspace_name: String,
    role: String,
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
/// `activate: false` files the key WITHOUT making it the active credential — the compatibility guard uses it to keep a
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

/// Where a login-time mint should bind when the login's *organization*
/// names no workspace of its own.
///
/// The login plane resolves an untargeted mint from the JWT's `org_id`
/// claim and refuses when that organization has no (non-draft) Hedra
/// workspace. That refusal is not the end of the story: the account can
/// still be a member of workspaces the organization does not name — an
/// org-less "born-free" workspace has no organization to log in with at
/// all — and the mint accepts an explicit `workspace_id` that bypasses the
/// org claim entirely. This is the choice of which one to name.
///
/// Deliberately never guesses between several. Binding a key also binds
/// what it bills, so a tie the user has not already broken is reported
/// rather than resolved.
pub(crate) enum FallbackTarget {
    /// One defensible target — name it on the retry.
    One {
        workspace_id: String,
        workspace_name: String,
    },
    /// The account reaches no workspace at all. There is nothing to fall
    /// back to, and the server's refusal is simply the truth.
    Nothing,
    /// Several reachable workspaces and no prior selection to break the
    /// tie. Each entry is a display line, `<id> — <name> (<role>)`.
    Ambiguous(Vec<String>),
}

/// Pick the workspace a refused login-time mint should retry against.
///
/// Precedence:
///   1. the workspace already selected locally, when the listing still
///      shows it — `workspaces select` is an explicit choice, and a login
///      must not silently move the active key off it;
///   2. the only workspace in the listing, which is no guess at all;
///   3. otherwise nothing is picked.
///
/// A stale selection — one no longer in the listing, after access was
/// revoked or a workspace was left — falls through to 2 rather than being
/// retried: naming it would only earn a membership refusal.
pub(crate) fn fallback_mint_target(cli_name: &str, listing: &[WorkspaceSummary]) -> FallbackTarget {
    fn one(ws: &WorkspaceSummary) -> FallbackTarget {
        FallbackTarget::One {
            workspace_id: ws.workspace_id.clone(),
            workspace_name: ws.workspace_name.clone(),
        }
    }

    let selected = WorkspaceKeyMap::load(cli_name).active_workspace_id;
    if let Some(ws) = selected
        .as_deref()
        .and_then(|id| listing.iter().find(|w| w.workspace_id == id))
    {
        return one(ws);
    }
    match listing {
        [] => FallbackTarget::Nothing,
        [only] => one(only),
        several => FallbackTarget::Ambiguous(
            several
                .iter()
                .map(|w| format!("{} — {} ({})", w.workspace_id, w.workspace_name, w.role))
                .collect(),
        ),
    }
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
    // Saving the map is also what sheds any legacy standalone `KeyAuth`
    // item — see `active_key::write_map`. No separate cleanup call: routing
    // one through the projection would now delete the map itself.
    map.save(cli_name)?;
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
/// same JWT still covers the mint that may follow, so this does not need a
/// login of its own.
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

// ---------------------------------------------------------------------------
// Command output: the standard pipeline, same as every generated command.
// ---------------------------------------------------------------------------

/// The `workspaces list` payload, in the list-shaped envelope the generated
/// commands use (`{"data": [...]}`) so `OutputPipeline` renders it the same
/// way — table on a TTY, JSON when piped, plus `--format`, `--query` and
/// `--quiet`.
///
/// `active` is a string — [`ACTIVE_MARKER`] on the active row, empty
/// everywhere else — rather than a boolean, so the column reads as a marker
/// in the table instead of a column of `true`/`false`. Filter it with
/// `--query "data[?active != '']"`.
///
/// `api_key` carries the held key's id — the non-secret half of the
/// `<key_id>:<secret>` credential, which is what this scheme's "key prefix"
/// is — or `null` where no key is held. It replaced a `key_held` boolean:
/// presence answers the same "can `select` switch to this one offline?"
/// question, and naming the key also says *which* key that would be.
/// `--query 'data[?api_key]'` keeps the boolean's filtering behaviour,
/// since JMESPath counts `null` as false.
///
/// The secret half is never carried here — see [`HeldKey::credential`].
///
/// `workos_organization_id` is deliberately not carried: it is an
/// identifier for the upstream identity provider, not something a caller
/// picks a workspace by, and as a column it cost more width than any other
/// field while being empty for personal workspaces. Nothing renders it any
/// more, so `WorkspaceSummary` no longer parses it either.
///
/// Column order is alphabetical and is not a lever: `serde_json` is
/// BTreeMap-backed in this workspace (the `preserve_order` feature is off),
/// so key insertion order here has no effect on the rendered order. Renaming
/// a field is the only way to move a column.
pub(crate) fn workspace_rows(
    workspaces: &[WorkspaceSummary],
    active_id: Option<&str>,
    held: &BTreeMap<String, HeldKey>,
) -> Value {
    let rows: Vec<Value> = workspaces
        .iter()
        .map(|ws| {
            json!({
                "active": active_marker(active_id == Some(ws.workspace_id.as_str())),
                "api_key": held.get(&ws.workspace_id).map(|k| k.key_id.as_str()),
                "role": ws.role,
                "workspace_id": ws.workspace_id,
                "workspace_name": ws.workspace_name,
            })
        })
        .collect();
    json!({ "data": rows })
}

/// What the `active` column holds on the active row. Empty string
/// otherwise — an absent marker, not `false`.
pub(crate) const ACTIVE_MARKER: &str = "*";

fn active_marker(active: bool) -> &'static str {
    if active {
        ACTIVE_MARKER
    } else {
        ""
    }
}

/// The `workspaces select` result: the workspace that is now active, as one
/// object through the same pipeline the listing uses. Same `active` marker
/// and same `api_key` / `workspace_id` / `workspace_name` spellings as a
/// listing row, so the two line up column-for-column.
///
/// Deliberately never carries [`HeldKey::credential`] — that is the live
/// `<key_id>:<secret>` pair, and this value goes to stdout.
fn selected_row(workspace_id: &str, name: Option<&str>, key_id: Option<&str>) -> Value {
    json!({
        // `select` succeeded, so this workspace is by definition the active
        // one — the marker is not conditional here.
        "active": ACTIVE_MARKER,
        "api_key": key_id,
        "workspace_id": workspace_id,
        "workspace_name": name,
    })
}

/// The listing exactly as `workspaces list` draws it at `--format table`.
///
/// Login pins the table format rather than building a pipeline: it is not a
/// data-emitting command, so there is no `--format` to honor and its output
/// is not pipeable. Everything else — columns, the `active` marker, the
/// trailing note — is the command's own rendering, so what you see after a
/// login is what `workspaces list` would print.
fn render_listing_table(
    workspaces: &[WorkspaceSummary],
    active_id: Option<&str>,
    held: &BTreeMap<String, HeldKey>,
) -> String {
    let note = listing_note(workspaces, active_id);
    // An empty listing has no rows to tabulate; the note is the whole story.
    if workspaces.is_empty() {
        return format!("{}\n", note.unwrap_or_default());
    }
    let table = format_value(
        &workspace_rows(workspaces, active_id, held),
        &OutputFormat::Table,
    );
    match note {
        Some(n) => format!("{table}{n}\n"),
        None => table,
    }
}

/// Render `value` to stdout through the standard pipeline.
///
/// `matches` must be the deepest `ArgMatches` in play: `--format`,
/// `--query` and `--quiet` are declared `.global(true)` on the root, so
/// clap propagates them to every level, and the deepest match is the one
/// guaranteed to carry a value supplied after the subcommand.
fn emit(matches: &clap::ArgMatches, cli_name: &str, value: &Value) -> Result<(), CliError> {
    let pipeline = OutputPipeline::from_matches(matches, cli_name)
        .map_err(|e| CliError::Validation(e.to_string()))?;
    let mut out = std::io::stdout().lock();
    pipeline
        .emit(&mut out, value, false, true)
        .context("Failed to write output")?;
    Ok(())
}

/// The one line of context a listing sometimes needs but cannot express as
/// a row. `None` when the listing speaks for itself.
///
/// Shared so the command and the login display cannot drift: the command
/// prints it on stderr (below), login appends it under the table.
fn listing_note(workspaces: &[WorkspaceSummary], active_id: Option<&str>) -> Option<String> {
    if workspaces.is_empty() {
        return Some("No workspaces visible to this account.".to_string());
    }
    match active_id {
        Some(id) if !workspaces.iter().any(|w| w.workspace_id == id) => Some(format!(
            "(active key is bound to workspace {id}, which is not in this listing)"
        )),
        None => Some("(no workspace-bound key is active)".to_string()),
        _ => None,
    }
}

/// Context the old text renderer carried inline, now on stderr so stdout
/// stays pure data (the same split `warn_if_env_key_shadows` already uses).
fn list_notes(workspaces: &[WorkspaceSummary], active_id: Option<&str>) {
    if let Some(note) = listing_note(workspaces, active_id) {
        eprintln!("{note}");
    }
}

/// What the post-login summary prints on stderr once the browser
/// round-trip completes (see `auth::bootstrap_inner`).
///
/// A table earns its header row only when there is a choice to look at. With
/// a single workspace there is nothing to compare and nothing to switch to,
/// so six columns of chrome say less than one line naming it; with none, the
/// note is the whole story. Past that it is the `workspaces list` table
/// verbatim, so the two views cannot drift apart.
pub(crate) fn render_login_summary(
    workspaces: &[WorkspaceSummary],
    active_id: Option<&str>,
    held: &BTreeMap<String, HeldKey>,
) -> String {
    match workspaces {
        [] => format!(
            "{}\n",
            listing_note(workspaces, active_id).unwrap_or_default()
        ),
        [only] => {
            let line = format!(
                "Workspace: {} ({})\n",
                only.workspace_name, only.workspace_id
            );
            // Still worth saying when the key that was just bootstrapped is
            // not bound to the one workspace this account can see.
            match listing_note(workspaces, active_id) {
                Some(note) => format!("{line}{note}\n"),
                None => line,
            }
        }
        _ => render_listing_table(workspaces, active_id, held),
    }
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
                    "Switch the active credential to the key for the given \
                     workspace. A held key is renewed first, which both proves it \
                     still works and extends it; one that the API refuses is \
                     replaced, as is a workspace with no key held at all. All of \
                     that runs from the current login — no browser round-trip \
                     unless you are not logged in. Works for workspaces with no \
                     linked organization, which no login can select by itself.",
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
    let app = ctx.downcast_ref::<AppContext>().ok_or_else(|| {
        CliError::Validation("workspaces: unexpected binding context type".to_string())
    })?;
    let cli_name = app.http_config().name().to_string();
    // The login plane follows `--base-url` / `HEDRA_CLI_BASE_URL` like every
    // generated command does. Reading it from the context rather than the
    // environment is what picks up the flag, which never reaches env.
    let api_base = match app.base_url_override() {
        Some(raw) => auth::resource_base_from_override(raw)?,
        None => auth::resource_base()?,
    };
    // Pass the deepest matches on to the handler: global flags propagate to
    // every level, but only the deepest is certain to carry a value written
    // after the subcommand (`workspaces list --format json`).
    match matches.subcommand() {
        None => run_list(&cli_name, &api_base, matches),
        Some(("list", sub)) => run_list(&cli_name, &api_base, sub),
        Some(("select", sub)) => {
            let ws = sub
                .get_one::<String>("workspace-id")
                .expect("--workspace-id is required");
            run_select(&cli_name, &api_base, ws, sub)
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

/// Fetch the listing with an already-fresh login JWT. `/v3/workspaces` is a
/// login-plane endpoint, so `jwt` must be one `auth::fresh_login_jwt`
/// produced — an unexpired token from the keyring is not enough.
///
/// The token is a parameter rather than minted here so a caller needing two
/// login-plane calls pays for one rotation, not two.
fn fetch_listing_with_jwt(api_base: &str, jwt: &str) -> Result<Vec<WorkspaceSummary>, CliError> {
    let http = http_client()?;
    let url = format!("{api_base}/v3/workspaces");
    run_async(fetch_workspaces_request(http.get(url).bearer_auth(jwt)))
}

fn warn_if_env_key_shadows() {
    if std::env::var("HEDRA_API_KEY").is_ok() {
        eprintln!("⚠ HEDRA_API_KEY is set and shadows the keyring — the active workspace key is not what requests will use.");
    }
}

fn run_list(cli_name: &str, api_base: &str, matches: &clap::ArgMatches) -> Result<(), CliError> {
    let jwt = auth::fresh_login_jwt(cli_name)?;
    let listing = fetch_listing_with_jwt(api_base, &jwt)?;
    let map = WorkspaceKeyMap::load(cli_name);
    let active = map.active_workspace_id.as_deref();
    emit(
        matches,
        cli_name,
        &workspace_rows(&listing, active, &map.keys),
    )?;
    list_notes(&listing, active);
    warn_if_env_key_shadows();
    Ok(())
}

fn run_select(
    cli_name: &str,
    api_base: &str,
    workspace_id: &str,
    matches: &clap::ArgMatches,
) -> Result<(), CliError> {
    // A held key is looked up but NOT trusted. Activating one without
    // checking it made "the CLI says I am on this workspace" and "the CLI
    // can call this workspace" two different facts: a revoked key earned a
    // green marker and then a 401 on every request, with no way back short
    // of `auth logout`. So the marker moves only once the key is known good
    // — or once a replacement has been minted.
    let held = WorkspaceKeyMap::load(cli_name)
        .keys
        .get(workspace_id)
        .cloned();

    // Everything that can verify or replace a key needs a login session.
    // With a key in hand but no session, switching to it unverified still
    // beats refusing to switch: the key is probably fine, this is what the
    // command did before it verified anything, and the note names the fix
    // if it turns out not to be.
    if !auth::has_oauth_session(cli_name) {
        if let Some(key) = &held {
            return activate_unverified(
                matches,
                cli_name,
                workspace_id,
                key,
                "not logged in",
            );
        }
        eprintln!("Not logged in — launching browser login…");
        auth::EnvPkceLoginFlow::new().run(&LoginContext {
            cli_name: cli_name.to_string(),
            no_browser: false,
        })?;
        // The login's own bootstrap may already have landed on the target,
        // and it verified what it left behind — renewing or minting — so
        // this activation needs no check of its own.
        if let SelectOutcome::Activated(key) = activate(cli_name, workspace_id)? {
            announce_active(
                matches,
                cli_name,
                workspace_id,
                key.workspace_name.as_deref(),
                Some(&key.key_id),
            )?;
            return Ok(());
        }
    }

    // One forced refresh for the whole command — it covers the renewal, the
    // listing and the mint below. Each refresh rotates the token
    // server-side, so refreshing again for a later leg would invalidate the
    // one just used, and if the first rotation failed to persist, the second
    // would present a dead token and fail outright.
    let jwt = match auth::fresh_login_jwt(cli_name) {
        Ok(jwt) => jwt,
        // A session that cannot be refreshed must not make a held key
        // unreachable: before verification existed this command never
        // needed the network at all, and a stale session is no reason to
        // refuse a switch.
        Err(_) if held.is_some() => {
            let key = held.as_ref().expect("held is Some in this arm");
            return activate_unverified(
                matches,
                cli_name,
                workspace_id,
                key,
                "the login session could not be refreshed",
            );
        }
        Err(e) => return Err(e),
    };

    // Renew rather than trust. The renewal is what proves the key still
    // works, and it extends it in the same round-trip; a refusal is exactly
    // the signal that it has to be replaced, and falls through to the mint
    // below (which reports itself — `try_renew` already said why).
    if let Some(key) = &held {
        if let Some(expires_at) =
            auth::renew_held_key_at(cli_name, api_base, &jwt, workspace_id, &key.credential)?
        {
            announce_active(
                matches,
                cli_name,
                workspace_id,
                key.workspace_name.as_deref(),
                Some(&key.key_id),
            )?;
            eprintln!("(key {} verified and renewed — {expires_at})", key.key_id);
            return Ok(());
        }
    }

    // Fail on a typo'd or invisible id before minting anything; the listing
    // also supplies the display name for the confirmation line.
    let listing = fetch_listing_with_jwt(api_base, &jwt)?;
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
        "{} for {} ({workspace_id}) — minting one…",
        if held.is_some() {
            "The held API key is no longer usable"
        } else {
            "No API key held"
        },
        target.workspace_name
    );
    let minted = auth::mint_for_workspace_at(cli_name, api_base, &jwt, workspace_id)?;
    announce_active(
        matches,
        cli_name,
        workspace_id,
        minted
            .workspace_name
            .as_deref()
            .or(Some(target.workspace_name.as_str())),
        Some(&minted.key_id),
    )?;
    if let Some(expiry) = minted.expires_at.as_deref() {
        eprintln!("(key expires {expiry})");
    }
    Ok(())
}

/// Switch to a held key that could not be checked, and say so.
///
/// Reached only when the login plane is out of reach — no session, or one
/// that will not refresh — so there is no way to renew the key or mint a
/// replacement. Switching anyway is the lesser evil: it is what this command
/// did before it verified anything, the key is very likely live, and a
/// refusal would strand a user who is merely offline. The note is what keeps
/// the silent-401 failure from coming back: it says the marker moved on
/// trust, and names the command that would settle it.
fn activate_unverified(
    matches: &clap::ArgMatches,
    cli_name: &str,
    workspace_id: &str,
    key: &HeldKey,
    why: &str,
) -> Result<(), CliError> {
    activate(cli_name, workspace_id)?;
    eprintln!(
        "({why} — key {} was activated without being verified; run \
         `hedra-cli auth login` if requests come back 401)",
        key.key_id
    );
    announce_active(
        matches,
        cli_name,
        workspace_id,
        key.workspace_name.as_deref(),
        Some(&key.key_id),
    )
}

/// Report the newly active workspace through the standard pipeline, so
/// `select` honors `--format` / `--query` / `--quiet` exactly like the
/// listing and every generated command.
fn announce_active(
    matches: &clap::ArgMatches,
    cli_name: &str,
    workspace_id: &str,
    name: Option<&str>,
    key_id: Option<&str>,
) -> Result<(), CliError> {
    emit(matches, cli_name, &selected_row(workspace_id, name, key_id))?;
    warn_if_env_key_shadows();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fern_cli_sdk::auth::{KeyringStore, MockKeyringStore};

    fn ws(id: &str, name: &str) -> WorkspaceSummary {
        WorkspaceSummary {
            workspace_id: id.to_string(),
            workspace_name: name.to_string(),
            role: "member".to_string(),
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

    // ── command output (the standard pipeline path) ─────────────────────

    #[test]
    fn workspace_rows_are_list_shaped_with_the_markers_as_fields() {
        let list = [ws("w1", "Personal"), ws("w2", "Acme"), ws("w3", "Big Corp")];
        let mut held_keys = BTreeMap::new();
        held_keys.insert("w1".to_string(), held("key_1"));
        held_keys.insert("w2".to_string(), held("key_2"));

        let out = workspace_rows(&list, Some("w2"), &held_keys);

        // The `{"data": [...]}` envelope is what makes the shared formatter
        // render this as a table of rows rather than one key/value blob.
        let rows = out["data"].as_array().expect("list-shaped envelope");
        assert_eq!(rows.len(), 3);

        // The markers survive as real fields, not decoration. `active` is a
        // marker string so the column reads as `*` / blank rather than as a
        // column of `true`/`false`; `api_key` names the held key instead of
        // just asserting that one exists.
        assert_eq!(rows[1]["workspace_name"], "Acme");
        assert_eq!(rows[1]["active"], ACTIVE_MARKER);
        assert_eq!(rows[1]["api_key"], "key_2");

        assert_eq!(rows[0]["active"], "");
        assert_eq!(rows[0]["api_key"], "key_1");

        // No key held: a real absence, which the table renders as a blank
        // cell and `--query 'data[?api_key]'` filters out.
        assert_eq!(rows[2]["active"], "");
        assert!(rows[2]["api_key"].is_null());

        // Only the id half ever appears — never the secret behind the colon.
        let serialized = serde_json::to_string(&out).unwrap();
        assert!(
            !serialized.contains("secret"),
            "credential leaked into the listing: {serialized}"
        );

        // Exactly one row may carry the marker.
        assert_eq!(
            rows.iter().filter(|r| r["active"] == ACTIVE_MARKER).count(),
            1
        );

        // The upstream identity-provider id is deliberately not a column —
        // widest field in the table, empty for personal workspaces, and not
        // how anyone selects a workspace.
        for row in rows {
            assert!(
                row.get("workos_organization_id").is_none(),
                "org id leaked back into the listing: {row}"
            );
        }
    }

    #[test]
    fn workspace_rows_empty_listing_keeps_the_envelope() {
        // Scripts still get well-formed `{"data": []}`; the human-facing
        // "no workspaces" line is `list_notes`' job, on stderr.
        let out = workspace_rows(&[], None, &BTreeMap::new());
        assert_eq!(out["data"].as_array().map(Vec::len), Some(0));
    }

    // The point of the change: the shared formatter, not a hand-rolled
    // renderer, draws the listing — so `--format table` gets a real header +
    // separator + aligned columns, and every other format works for free.
    #[test]
    fn the_shared_formatter_draws_a_real_table_and_real_json() {
        use fern_cli_sdk::formatter::{format_value, OutputFormat};

        let list = [ws("w1", "Personal"), ws("w2", "Acme")];
        let mut held_keys = BTreeMap::new();
        held_keys.insert("w2".to_string(), held("key_2"));
        let value = workspace_rows(&list, Some("w2"), &held_keys);

        let table = format_value(&value, &OutputFormat::Table);
        let mut lines = table.lines();
        let header = lines.next().expect("header row");
        for column in [
            "active",
            "api_key",
            "role",
            "workspace_id",
            "workspace_name",
        ] {
            assert!(header.contains(column), "missing {column} in: {header}");
        }
        assert!(
            !header.contains("workos"),
            "org id is not a column: {header}"
        );
        assert!(
            lines.next().is_some_and(|l| l.starts_with('─')),
            "no separator rule under the header: {table}"
        );
        let acme = table.lines().find(|l| l.contains("Acme")).unwrap();
        assert!(acme.contains("w2") && acme.contains("member"));
        // Rows are padded to a common width — that is the "aligned table"
        // the bespoke renderer used to hand-roll.
        let widths: Vec<usize> = table
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.chars().count())
            .collect();
        assert!(
            widths.windows(2).all(|w| w[0] == w[1]),
            "ragged rows: {widths:?}"
        );

        let json: serde_json::Value =
            serde_json::from_str(&format_value(&value, &OutputFormat::Json)).unwrap();
        assert_eq!(json, value);
    }

    #[test]
    fn global_output_flags_reach_the_matches_dispatch_hands_to_handlers() {
        // The whole change rests on this: `--format` / `--query` / `--quiet`
        // are declared `.global(true)` on the root the SDK builds, so a
        // custom command registered on that root can read them back with
        // `OutputPipeline::from_matches`. Built against the real root rather
        // than a hand-rolled stand-in, so it fails if the SDK ever stops
        // declaring them global — which would silently strand `workspaces`
        // on whatever the TTY default happened to be.
        use fern_cli_sdk::formatter::OutputFormat;
        use fern_cli_sdk::openapi::{commands, load_openapi_spec};

        let doc = load_openapi_spec(include_str!("openapi0.json"), "hedra-cli")
            .expect("the bundled spec parses");
        let root = commands::build_cli(&doc).subcommand(command());

        // Written after the subcommand — the position `dispatch`'s
        // deepest-matches choice exists to handle.
        let m = root
            .clone()
            .try_get_matches_from(["hedra-cli", "workspaces", "list", "--format", "yaml"])
            .expect("--format is accepted after the subcommand");
        let Some(("workspaces", ws)) = m.subcommand() else {
            panic!("expected workspaces")
        };
        let Some(("list", sub)) = ws.subcommand() else {
            panic!("expected list")
        };
        assert_eq!(
            OutputPipeline::from_matches(sub, "hedra-cli")
                .unwrap()
                .format,
            OutputFormat::Yaml
        );

        // And written before it, which clap propagates downward.
        let m = root
            .clone()
            .try_get_matches_from([
                "hedra-cli",
                "--format",
                "csv",
                "--quiet",
                "workspaces",
                "list",
            ])
            .expect("globals are accepted before the subcommand");
        let Some(("workspaces", ws)) = m.subcommand() else {
            panic!("expected workspaces")
        };
        let Some(("list", sub)) = ws.subcommand() else {
            panic!("expected list")
        };
        let pipeline = OutputPipeline::from_matches(sub, "hedra-cli").unwrap();
        assert_eq!(pipeline.format, OutputFormat::Csv);
        assert!(pipeline.quiet);

        // Bare `workspaces` (no subcommand) still carries them — that path
        // hands `dispatch`'s own matches to `run_list`.
        let m = root
            .clone()
            .try_get_matches_from(["hedra-cli", "workspaces", "--format", "jsonl"])
            .expect("bare workspaces accepts globals");
        let Some(("workspaces", ws)) = m.subcommand() else {
            panic!("expected workspaces")
        };
        assert!(ws.subcommand().is_none());
        assert_eq!(
            OutputPipeline::from_matches(ws, "hedra-cli")
                .unwrap()
                .format,
            OutputFormat::Jsonl
        );
    }

    // The doc comment on `workspace_rows` advertises these two filters.
    // Pin them so the advice cannot rot.
    #[test]
    fn the_documented_query_filters_work() {
        let list = [ws("w1", "Personal"), ws("w2", "Acme"), ws("w3", "Big Corp")];
        let mut held_keys = BTreeMap::new();
        held_keys.insert("w1".to_string(), held("key_1"));
        held_keys.insert("w2".to_string(), held("key_2"));
        let v = workspace_rows(&list, Some("w2"), &held_keys);

        let run = |expr: &str| -> Vec<String> {
            let compiled = jmespath::compile(expr).expect("expression compiles");
            let data = jmespath::Variable::from_json(&serde_json::to_string(&v).unwrap()).unwrap();
            let out = compiled.search(data).expect("expression evaluates");
            serde_json::from_str::<Vec<serde_json::Value>>(&serde_json::to_string(&out).unwrap())
                .expect("an array of rows")
                .iter()
                .map(|r| r["workspace_name"].as_str().unwrap().to_string())
                .collect()
        };

        assert_eq!(run("data[?active != '']"), ["Acme"]);
        assert_eq!(run("data[?api_key]"), ["Personal", "Acme"]);
    }

    /// The fallback names the sole workspace, and only the sole one — with
    /// two in play and nothing selected, guessing would bind this device's
    /// billing to a workspace the user never chose.
    #[test]
    #[serial_test::serial]
    fn fallback_target_takes_the_only_workspace_but_never_guesses() {
        let _store = fresh_keyring();

        assert!(matches!(
            fallback_mint_target("test-cli", &[]),
            FallbackTarget::Nothing
        ));
        assert!(matches!(
            fallback_mint_target("test-cli", &[ws("w1", "Personal")]),
            FallbackTarget::One { ref workspace_id, ref workspace_name }
                if workspace_id == "w1" && workspace_name == "Personal"
        ));

        let FallbackTarget::Ambiguous(candidates) =
            fallback_mint_target("test-cli", &[ws("w1", "Personal"), ws("w2", "Acme")])
        else {
            panic!("two workspaces and no selection must not resolve to one");
        };
        assert_eq!(candidates.len(), 2);
        // Each line has to carry the id, because the id is what the user
        // then passes to `workspaces select`.
        assert!(candidates[0].contains("w1") && candidates[0].contains("Personal"));
        assert!(candidates[1].contains("w2") && candidates[1].contains("Acme"));
    }

    /// A prior `workspaces select` breaks the tie — but only while the
    /// listing still shows it. A selection pointing at a workspace the
    /// account can no longer reach (access revoked, or written by an older
    /// CLI) must not be retried: naming it would only earn a membership
    /// refusal, so it falls through to the sole-workspace rule.
    #[test]
    #[serial_test::serial]
    fn fallback_target_prefers_a_live_selection_and_ignores_a_stale_one() {
        let _store = fresh_keyring();
        let listing = [ws("w1", "Personal"), ws("w2", "Acme")];

        record_key(
            "test-cli",
            Some("w2"),
            "k2",
            "k2:s",
            Some("Acme"),
            None,
            true,
        )
        .unwrap();
        assert!(matches!(
            fallback_mint_target("test-cli", &listing),
            FallbackTarget::One { ref workspace_id, .. } if workspace_id == "w2"
        ));

        // Selected a workspace that is no longer listed at all.
        record_key("test-cli", Some("w9"), "k9", "k9:s", None, None, true).unwrap();
        assert!(
            matches!(
                fallback_mint_target("test-cli", &listing),
                FallbackTarget::Ambiguous(_)
            ),
            "a stale selection must not be treated as a live choice"
        );
        // …and with one workspace left, the stale selection still yields to it.
        assert!(matches!(
            fallback_mint_target("test-cli", &listing[..1]),
            FallbackTarget::One { ref workspace_id, .. } if workspace_id == "w1"
        ));
    }

    #[test]
    fn list_notes_stay_off_stdout() {
        // Nothing here writes to stdout — the notes are stderr-only so that
        // `--format json` output stays parseable. Exercised for panics and
        // for the branch shape.
        list_notes(&[], None);
        list_notes(&[ws("w1", "A")], None);
        list_notes(&[ws("w1", "A")], Some("w9"));
        list_notes(&[ws("w1", "A")], Some("w1"));
    }

    #[test]
    fn selected_row_is_an_object_that_lines_up_with_a_listing_row() {
        let out = selected_row("w2", Some("Acme"), Some("key_2"));

        assert_eq!(out["workspace_id"], "w2");
        assert_eq!(out["workspace_name"], "Acme");
        assert_eq!(out["api_key"], "key_2");
        // `select` just made this workspace active, so the marker is not
        // conditional — and it is spelled the same way a listing row spells
        // it, so the two render as the same column.
        assert_eq!(out["active"], ACTIVE_MARKER);

        // Field names match the listing's, so `select` and `list` agree.
        let list = [ws("w2", "Acme")];
        let mut held_keys = BTreeMap::new();
        held_keys.insert("w2".to_string(), held("key_2"));
        let row = workspace_rows(&list, Some("w2"), &held_keys)["data"][0].clone();
        for key in ["active", "api_key", "workspace_id", "workspace_name"] {
            assert_eq!(out[key], row[key], "`{key}` disagrees between the two");
        }

        // An unknown name is a real absence, not the string "<unnamed>".
        assert!(selected_row("w2", None, None)["workspace_name"].is_null());
    }

    #[test]
    fn selected_row_never_leaks_the_credential() {
        // This value goes to stdout, and `HeldKey::credential` is the live
        // `<key_id>:<secret>` pair. It must never appear there.
        let key = held("key_1");
        let out = selected_row("w2", key.workspace_name.as_deref(), Some(&key.key_id));
        let serialized = serde_json::to_string(&out).unwrap();
        assert!(
            !serialized.contains("secret"),
            "credential leaked into stdout payload: {serialized}"
        );
    }

    // The post-login display is the listing, not a second rendering that can
    // drift away from it.
    #[test]
    fn the_login_display_is_the_listing_table() {
        use fern_cli_sdk::formatter::{format_value, OutputFormat};

        let list = [ws("w1", "Personal"), ws("w2", "Acme")];
        let mut held_keys = BTreeMap::new();
        held_keys.insert("w2".to_string(), held("key_2"));

        let login = render_listing_table(&list, Some("w2"), &held_keys);
        let command = format_value(
            &workspace_rows(&list, Some("w2"), &held_keys),
            &OutputFormat::Table,
        );
        assert_eq!(
            login, command,
            "login drew something other than the listing table"
        );
        assert!(login.contains(ACTIVE_MARKER) && login.contains("workspace_name"));

        // The note rides along under the table when there is one. `w9` is not
        // in the listing, so no row carries the marker — the table it sits
        // under is that listing's own table, not the one above.
        let unlisted = render_listing_table(&list, Some("w9"), &held_keys);
        let unlisted_table = format_value(
            &workspace_rows(&list, Some("w9"), &held_keys),
            &OutputFormat::Table,
        );
        assert!(
            unlisted.starts_with(&unlisted_table),
            "note replaced the table instead of following it: {unlisted}"
        );
        assert!(!unlisted_table.contains(ACTIVE_MARKER), "unexpected marker");
        assert!(unlisted.trim_end().ends_with("not in this listing)"));
        assert!(render_listing_table(&list, None, &held_keys)
            .contains("no workspace-bound key is active"));

        // Nothing to tabulate: the note is the whole output.
        assert_eq!(
            render_listing_table(&[], None, &BTreeMap::new()),
            "No workspaces visible to this account.\n"
        );
    }

    #[test]
    fn the_login_summary_only_tabulates_when_there_is_a_choice() {
        let mut held_keys = BTreeMap::new();
        held_keys.insert("w1".to_string(), held("key_1"));

        // One workspace: a one-row table is more chrome than content, so it
        // is named on one line instead — no headers, no separator rule.
        let one = render_login_summary(&[ws("w1", "Personal")], Some("w1"), &held_keys);
        assert_eq!(one, "Workspace: Personal (w1)\n");

        // Two: the listing table, byte-for-byte what `workspaces list` draws.
        let list = [ws("w1", "Personal"), ws("w2", "Acme")];
        let two = render_login_summary(&list, Some("w1"), &held_keys);
        assert_eq!(two, render_listing_table(&list, Some("w1"), &held_keys));
        assert!(two.contains("workspace_name") && two.contains(ACTIVE_MARKER));

        // None: the note is all there is.
        assert_eq!(
            render_login_summary(&[], None, &BTreeMap::new()),
            "No workspaces visible to this account.\n"
        );
    }

    #[test]
    fn the_login_summary_still_flags_a_key_bound_elsewhere() {
        // The single-workspace line has no `active` column to carry this, so
        // the note has to survive the shortened form.
        let out = render_login_summary(&[ws("w1", "Personal")], Some("w9"), &BTreeMap::new());
        assert!(out.starts_with("Workspace: Personal (w1)\n"), "{out}");
        assert!(out.trim_end().ends_with("not in this listing)"), "{out}");
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

    use wiremock::matchers::{body_partial_json, header, method, path};
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

    /// A `select` that has to mint makes two login-plane calls — the listing
    /// and the mint — and must pay for exactly one token rotation between
    /// them.
    ///
    /// It used to force a refresh for each. Every exchange rotates the
    /// refresh token server-side, so the second one invalidated the token
    /// the first had just stored; and if the first rotation failed to
    /// persist, the second presented a dead token and the command failed
    /// outright. Two concurrent CLI processes raced the same way.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn select_refreshes_the_session_only_once() {
        use super::super::auth::tests as auth_tests;

        auth_tests::clear_endpoint_override();
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/workspaces"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"workspace_id": "w2", "workspace_name": "Acme", "role": "admin",
                          "workos_organization_id": "org_1"}],
                "next_cursor": null,
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "key_id": "key_w2", "credential": "key_w2:s3cret", "kind": "personal",
                "workspace_id": "w2", "workspace_name": "Acme", "organization_id": "org_1",
                "expires_at": "2026-08-21T00:00:00Z",
            })))
            .mount(&server)
            .await;

        let store = fresh_keyring();
        let _home = auth_tests::seed_live_session(&store, &server.uri());
        // `.expect(1)` is the assertion: wiremock verifies it on drop.
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "fresh-jwt", "refresh_token": "rotated",
                "token_type": "Bearer", "expires_in": 3600,
            })))
            .expect(1)
            .mount(&server)
            .await;

        let base = server.uri();
        let sub = select_matches("w2");
        tokio::task::spawn_blocking(move || run_select("test-cli", &base, "w2", &sub))
            .await
            .unwrap()
            .unwrap();

        let map = WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id.as_deref(), Some("w2"));
    }

    // ── select verifies the key it activates ────────────────────────────

    /// The deepest matches, as `dispatch` hands them over. Built off the
    /// real root CLI rather than `command()` alone: the output pipeline
    /// reads the SDK's *global* `--format`/`--quiet`, which the bare
    /// subcommand does not declare, and `OutputPipeline::from_matches`
    /// panics on a matches object that lacks them — so every `run_select`
    /// test has to go through the real rendering path.
    fn select_matches(workspace_id: &str) -> clap::ArgMatches {
        use fern_cli_sdk::openapi::{commands, load_openapi_spec};
        let doc = load_openapi_spec(include_str!("openapi0.json"), "hedra-cli")
            .expect("the bundled spec parses");
        let m = commands::build_cli(&doc)
            .subcommand(command())
            .try_get_matches_from([
                "hedra-cli",
                "workspaces",
                "select",
                "--workspace-id",
                workspace_id,
            ])
            .expect("select parses");
        m.subcommand_matches("workspaces")
            .and_then(|ws| ws.subcommand_matches("select"))
            .expect("deepest matches")
            .clone()
    }

    /// File a key for `workspace_id` and make it active, the state a prior
    /// `select` or login leaves behind.
    fn seed_held_key(workspace_id: &str, key_id: &str, credential: &str) {
        record_key(
            "test-cli",
            Some(workspace_id),
            key_id,
            credential,
            Some("Acme"),
            Some("2026-08-19T00:00:00Z"),
            true,
        )
        .unwrap();
    }

    async fn mock_token_endpoint(server: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "fresh-jwt", "refresh_token": "rotated",
                "token_type": "Bearer", "expires_in": 3600,
            })))
            .mount(server)
            .await;
    }

    async fn run_select_against(base: String, workspace_id: &'static str) -> Result<(), CliError> {
        let sub = select_matches(workspace_id);
        tokio::task::spawn_blocking(move || run_select("test-cli", &base, workspace_id, &sub))
            .await
            .unwrap()
    }

    /// A held key is renewed, not trusted. The renewal doubles as the
    /// check — and it must NOT mint, because the key was fine.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn select_renews_a_held_key_rather_than_trusting_it() {
        use super::super::auth::tests as auth_tests;
        auth_tests::clear_endpoint_override();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap/renew"))
            .and(header("authorization", "Bearer key_w2:held"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "key_id": "key_w2", "expires_at": "2027-01-01T00:00:00Z", "workspace_id": "w2",
            })))
            .expect(1)
            .mount(&server)
            .await;
        // The assertion that matters as much as the renewal: a live key is
        // never replaced. wiremock verifies the count on drop.
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .respond_with(ResponseTemplate::new(201))
            .expect(0)
            .mount(&server)
            .await;
        let store = fresh_keyring();
        let _home = auth_tests::seed_live_session(&store, &server.uri());
        mock_token_endpoint(&server).await;
        seed_held_key("w2", "key_w2", "key_w2:held");

        run_select_against(server.uri(), "w2").await.unwrap();

        let map = WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id.as_deref(), Some("w2"));
        assert_eq!(
            map.keys["w2"].credential, "key_w2:held",
            "a renewal carries no new secret — the held credential stays"
        );
        assert_eq!(
            map.keys["w2"].expires_at.as_deref(),
            Some("2027-01-01T00:00:00Z"),
            "…but the extension has to be recorded, or the key was not really renewed"
        );
    }

    /// The reported defect: a held key that the login plane refuses used to
    /// be activated anyway, giving a green marker and then a 401 on every
    /// request. It must be replaced instead.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn select_remints_when_the_held_key_is_dead() {
        use super::super::auth::tests as auth_tests;
        auth_tests::clear_endpoint_override();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap/renew"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "error": {"code": "UNAUTHORIZED", "message": "Invalid API key."},
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/v3/workspaces"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [{"workspace_id": "w2", "workspace_name": "Acme", "role": "admin",
                          "workos_organization_id": "org_1"}],
                "next_cursor": null,
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .and(body_partial_json(serde_json::json!({"workspace_id": "w2"})))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "key_id": "key_new", "credential": "key_new:fresh", "kind": "personal",
                "workspace_id": "w2", "workspace_name": "Acme", "organization_id": "org_1",
                "expires_at": "2027-01-01T00:00:00Z",
            })))
            .expect(1)
            .mount(&server)
            .await;
        let store = fresh_keyring();
        let _home = auth_tests::seed_live_session(&store, &server.uri());
        mock_token_endpoint(&server).await;
        seed_held_key("w2", "key_dead", "key_dead:revoked");

        run_select_against(server.uri(), "w2").await.unwrap();

        let map = WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id.as_deref(), Some("w2"));
        assert_eq!(
            map.keys["w2"].credential, "key_new:fresh",
            "the dead credential must be replaced, not left active"
        );
        assert_eq!(
            active_store()
                .get("test-cli", super::super::auth::KEY_SCHEME)
                .unwrap()
                .as_deref(),
            Some("key_new:fresh"),
            "and the projection the SDK reads must follow"
        );
    }

    /// Verification needs the login plane. Offline, switching to the held
    /// key unverified still beats refusing to switch — but the user has to
    /// be told the marker moved on trust.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn select_activates_unverified_when_there_is_no_login_session() {
        use super::super::auth::tests as auth_tests;
        auth_tests::clear_endpoint_override();
        // No mocks at all: reaching the network would 404 and fail the run.
        let server = MockServer::start().await;
        let _store = fresh_keyring(); // no OAuth session seeded
        seed_held_key("w2", "key_w2", "key_w2:held");
        // `record_key` activated it; prove `select` is what re-activates.
        activate("test-cli", "w1").ok();

        run_select_against(server.uri(), "w2").await.unwrap();

        let map = WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id.as_deref(), Some("w2"));
        assert_eq!(map.keys["w2"].credential, "key_w2:held");
        assert!(
            server.received_requests().await.unwrap_or_default().is_empty(),
            "with no session there is nothing to ask the login plane"
        );
    }

    /// A session that will not refresh must not strand a held key either —
    /// this command needed no network at all before it verified anything.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn select_activates_unverified_when_the_session_cannot_refresh() {
        use super::super::auth::tests as auth_tests;
        auth_tests::clear_endpoint_override();
        let server = MockServer::start().await;
        // The refresh leg is mounted, and refuses.
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "error": "invalid_grant",
            })))
            .mount(&server)
            .await;
        let store = fresh_keyring();
        let _home = auth_tests::seed_live_session(&store, &server.uri());
        seed_held_key("w2", "key_w2", "key_w2:held");

        run_select_against(server.uri(), "w2").await.unwrap();

        let map = WorkspaceKeyMap::load("test-cli");
        assert_eq!(
            map.active_workspace_id.as_deref(),
            Some("w2"),
            "a stale session is no reason to refuse a switch"
        );
        assert_eq!(map.keys["w2"].credential, "key_w2:held");
    }
}

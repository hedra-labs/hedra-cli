# Credential storage contract

What the CLI stores, where, and what happens across versions. Written for
ENG-10414, which repaired a cluster of regressions caused by two parts of the
auth stack disagreeing about where the active credential lived.

## The one rule

**The workspace key map is the only place a data-plane credential lives.**

It occupies a single credential-store slot, `hedra-cli:WorkspaceKeys`, and
holds every API key the CLI has minted plus a marker naming the active one.
The `hedra-cli:KeyAuth` address that the SDK's injected keyring source reads
on every request is *projected* from that map at resolve time by
`cli/hedra-cli/active_key.rs` — it is not a second item holding a copy.

Everything else follows from that:

| Operation | Effect |
|---|---|
| `get(KeyAuth)` | the map's active credential; a legacy standalone item only if the map has no answer |
| `set(KeyAuth)` | writes into the map as `unbound_key` when a map exists, else a plain item |
| `delete(KeyAuth)` | clears the **whole map** and the legacy item |

The projection must stay symmetric. Reads and writes resolving to different
places is precisely what produced ENG-10414: `auth logout` deleted an item
that was not the credential in use and reported success while the CLI stayed
authenticated, and `auth login --with-token` wrote an item the projection then
ignored, so rotating a leaked key silently kept using the leaked one. Any
future change that projects one operation and not another reintroduces that
class of bug.

## Logout semantics

`auth logout` with no `--scheme` clears **every declared scheme** — the
data-plane key and the OAuth session both. The CLI declares two auth bindings
(an API-key scheme plus the OAuth login flow's), and requiring the user to
disambiguate between them made the bare command fail outright.

`auth logout --scheme KeyAuth` clears the entire workspace map, not just the
active entry. Every key in that map is a live credential the CLI can present
on demand, so leaving the non-active ones would let `workspaces select
<other>` re-authenticate with no challenge — "logged out" would be false for
anyone who has ever held a second workspace. Re-minting after a login is
cheap; a logout that leaves usable secrets on disk is not.

## Version compatibility

**Rolling back to v2.0.2 or earlier appears logged out. That is supported and
expected. Recovery is one `hedra-cli auth login`.**

Older binaries only understand the standalone `hedra-cli:KeyAuth` item; they
cannot read `WorkspaceKeys`. No compatibility mirror is written, and the
map-authoritative model ships without a migration window.

That is a deliberate choice, and it is cheap specifically because of *when*
it was made. The storage change landed in PR #102, which was never released —
`v2.0.2` is the newest tag and does not contain it. There is therefore no
deployed population with a half-migrated store to reason about: no shipped
binary has ever deleted a user's `KeyAuth` item, and no user has a map written
by a release. The alternative — reinstating the double write PR #102 removed —
would cost a second keychain item (its own macOS authorization prompt on every
`workspaces select`) permanently, to protect a downgrade path whose entire
cost is re-running one command.

A legacy item left by a pre-projection *development* build is still handled:
the map wins on read, and `active_key::write_map` sheds the item once the map
holds an active credential. It is deliberately **not** shed when a map write
files a key without activating it — the item is still what `get` falls back
to, so dropping it there would delete the credential the user is presenting.
The compatibility guard in `mint_for_workspace_at` produces exactly that
shape.

## Base URL

`--base-url` / `HEDRA_CLI_BASE_URL` names the **data-plane** root and carries
the `/v3` prefix, matching the spec's `https://api.hedra.com/v3` server. The
login plane derives its own origin from it by stripping that suffix, so both
planes always talk to the same deployment. An override without `/v3` is
refused before any request rather than guessed at.

This matters most for minting, the one login-plane call that *creates* state:
before ENG-10414 a `--base-url` pointing at a local stack still minted real
production API keys.

## SDK patches to re-apply after regeneration

`src/` is generator-owned. `.fernignore` protects `cli/hedra-cli/*.rs`,
`tests/`, `docs/`, `README.md` and friends — **not** `src/`. Fern Replay
records one patch per non-merge commit (`.fern/replay.lock` keys them by
`original_commit`) and replays them onto each new generation, but that has
failed before: commit `707fe94` is titled "re-apply the SDK header patches
after regeneration".

So each hand-written change to `src/` is kept in a commit that touches
**nothing else**, and listed here. After a regeneration, verify each is still
present; if not, `git cherry-pick <sha>`.

| sha | file | what |
|---|---|---|
| `d06b3c3` | `src/http.rs` | re-supply the `X-Fern-*` identity headers the CLI executor drops |
| `53c3773` | `src/http.rs` | send the `X-Fern-*` identity trio on the OpenAPI path |
| `8a3427d` | `src/pager.rs` | serialize PAGER env-var tests (see the `.fernignore` note — the file is deliberately not ignored) |
| `29c92df` | `src/auth/login.rs` | bare `auth logout` clears every declared scheme; `--with-token` pastes into the token binding rather than the OAuth one |

Keep this table current. A patch that is not listed is one nobody will notice
has been reverted.

**A rebase rewrites these shas.** They name commits on a branch, so rebasing
onto a moved trunk invalidates every row for work that has not merged yet —
silently, because a dangling sha looks exactly like a valid one until someone
tries to cherry-pick it. Re-read the table after any rebase of an open branch
and refresh the rows it covers.

# Wire-contract harness for hedra-cli outbound headers

**Date:** 2026-08-10
**Status:** Approved design
**Baseline:** `ed9886b` (main), all three crates at `1.0.0-dev`

## Problem

Nothing in this repository proves what a server actually receives from the CLI.
Three identity headers are declared in generated code, and static reading of the
*SDK's* call graph says none of the SDK's own copies reach the wire when an
executor is injected:

- `X-Fern-Language`, `X-Fern-SDK-Name`, `X-Fern-SDK-Version` are set in
  `ClientConfig::default()` (`hedra-sdk/src/config.rs:36-38`). They are applied
  only by `apply_custom_headers`.
- `X-Hedra-Spec-Version` is injected per-operation into
  `options.additional_headers` by every generated resource method (e.g.
  `hedra-sdk/src/api/resources/models/models.rs:45-51`). It is *also* applied
  only by `apply_custom_headers`.
- `send_request` (`hedra-sdk/src/core/http_client.rs:498-511`) branches on
  `self.executor`. The executor arm calls `executor.execute(req)` and never
  calls `apply_auth_headers` or `apply_custom_headers`. `execute_request`
  (line 297) builds the request with query params and body only.
- The CLI's sole SDK constructor (`cli/hedra/sdk.rs:48-56`) always injects an
  executor via `HttpClient::with_executor`.

So on the CLI's SDK path, all four of the SDK's own header copies are
constructed and then discarded. Note that this is `with_executor`'s documented
contract, not a latent SDK bug: its doc comment
(`hedra-sdk/src/core/http_client.rs:229-236`) says outright that "Auth headers,
custom headers, and retry logic are NOT applied by this client — the executor's
transport stack is expected to handle them", to avoid double-auth and
double-retry when the SDK is embedded in a CLI. The question this spec has to
answer is therefore not "does the SDK drop them" (yes, by design) but "does the
CLI's executor re-supply them" — and the answer differs per header.

### Correction (2026-08-11, pre-merge): ENG-10092 is NOT regressed

~~This defeats ENG-10092, which added `X-Hedra-Spec-Version` to the CLI surface
and was verified by grepping the *generated tree* — a check that cannot
distinguish "emitted" from "delivered".~~

**Retracted.** The reasoning above is sound about the SDK's copy and wrong about
the wire. `X-Hedra-Spec-Version` reaches the wire on a second, independent
channel that this analysis missed entirely — the CLI's own global-header
channel, which never passes through `apply_custom_headers`:

1. `cli/hedra/openapi0.json` declares
   `x-fern-global-headers: [{header: "X-Hedra-Spec-Version", name: "specVersion",
   optional: true, default: "3.2.2"}]`.
2. `src/openapi/app.rs:1568-1633` registers a hidden global clap arg per entry,
   with `.default_value("3.2.2")` — so it resolves on a bare invocation, with no
   flag and no env var.
3. `src/openapi/binding.rs:303-309` reads the matches back through
   `resolve_global_header_value` into `BindingEntry::global_headers`.
4. That same vector feeds *both* paths: `AppContext::new` for the OpenAPI
   dispatch path (`src/openapi/binding.rs:774`), and
   `AppContext::build_sdk_executor` (`src/openapi/app.rs:2288-2295`), which
   clones it into `CliExecutor::new`.
5. `CliExecutor::build_request` (`src/sdk_executor.rs:227-231`) stamps every pair
   onto every request it builds.

The original review missed this because the analysis stayed inside `hedra-sdk/`,
where the CLI's channel is invisible.

**What disproved it.** Two independent checks:

- *A trace of the real compiled binary.* `hedra --base-url http://127.0.0.1:8731
  models list`, against a header-logging server, sends:
  `authorization`, `accept`, **`x-hedra-spec-version: 3.2.2`**, `user-agent:
  hedra-cli/1.0.0-dev`, `accept-encoding`. Passing `--spec-version 9.9.9-proof`
  changes the header to `9.9.9-proof`, confirming the clap-flag → resolved
  `global_headers` → wire chain end to end, not merely a coincidental constant.
- *The harness itself, once corrected.* The failing assertion was an artifact of
  this harness: the replica passed `Vec::new()` as `CliExecutor`'s
  `global_headers` where the real bridge passes
  `entries[0].global_headers`. Supplying the real value flips
  `executor_path_preserves_spec_version_header` from red to green.

The `Vec::new()` came from the implementation plan, which prescribed it
literally; the implementers followed it correctly. The root cause was a
hand-simplified replica of a generated bridge, not regeneration drift — which is
why the replica now carries an explicit list of its divergences from the real
path rather than a general "keep this in sync" note.

That test is consequently **not** a defect report. It is now the repo's only
regression lock on ENG-10092, and it is worth keeping precisely because the
delivery depends on a channel a refactor could remove while the SDK-side copy
stays discarded.

The `X-Fern-*` trio is a different story and the finding there stands: nothing
in `cli/hedra/sdk.rs` merges `ClientConfig::custom_headers` into
`global_headers`, so those three genuinely never ship on the SDK path.

Separately, `X-Fern-SDK-Version` is a hardcoded `"0.1.0"` literal untouched
since the first commit. It survived `0.2.0 → 0.9.9 → 1.0.0-dev` and every
regeneration; `fern generate --version` has no channel to it. After #52
(ENG-10219) aligned all three manifests to `1.0.0-dev`, this constant is the
last place in the repo still claiming `0.1.0`.

Impact today is latent: `cli/hedra/custom.rs` is a 41-line template with every
example commented out, so nothing constructs the SDK client at runtime. Real
command traffic goes through `src/openapi/*` on `src/http.rs`'s own reqwest
client. ~~which sends only `User-Agent`.~~ **Corrected:** the traced binary sends
`authorization`, `accept`, `x-hedra-spec-version`, `user-agent`, and
`accept-encoding` — the "only `User-Agent`" claim came from reading
`HttpConfig::build_client`'s default-header set in isolation and missing the
headers the dispatch path stamps per request. What that path does *not* send is
the `X-Fern-*` trio, which is an SDK-side concept with no OpenAPI-path
equivalent. The remaining gap bites whoever writes the first custom command.

## Goals

Build an executable specification for the outbound wire contract that:

1. States what a server *should* receive, for each of the three ways a request
   can leave this CLI.
2. Fails today, precisely, once per distinct root cause — so the fix ticket
   cannot close half the problem and appear done. (Two such causes survived
   verification; a third turned out to be an artifact of the harness. See the
   correction above.)
3. Detects future drift automatically, without needing an edit at each release.

## Non-goals

- **Fixing the bypass.** Every file involved (`hedra-sdk/**`, `src/http.rs`,
  `cli/hedra/sdk.rs`) is generator-owned — absent from `.fernignore` — so an
  in-repo patch is reverted by the next regeneration. The durable fix belongs
  upstream in fern-config / fern-cli-generator and is filed separately.
- Asserting auth headers, retries, or TLS behavior.
- Driving the real compiled binary *as part of the committed suite*. Only the
  OpenAPI path is reachable that way, so it cannot exercise the SDK path this
  work is mostly about. ~~so it could not observe the headers this work is
  about.~~ **Corrected:** it observes more than assumed — the binary trace above
  is what disproved the `X-Hedra-Spec-Version` finding and the "only
  `User-Agent`" claim. A one-off binary trace is a cheap and unusually
  high-authority oracle, and skipping it is what let both errors stand; it is a
  non-goal for the *automated suite* (slow, needs a spawned process and a port),
  not for verification.

## Design

One integration test file, `tests/wire_contract.rs`, backed by `wiremock`
(already a dev-dependency at 0.6). No new dependencies.

Three scenarios, matching the three ways a request leaves this CLI:

| # | Path | Construction | Why it exists |
|---|------|--------------|---------------|
| 1 | OpenAPI | `HttpConfig::new("hedra").build_client()` | The only path real commands use today |
| 2 | SDK via executor | Replicates `cli/hedra/sdk.rs`: `CliExecutor::new(...)` → adapter → `HttpClient::with_executor` | What a custom command would hit |
| 3 | SDK direct | `HttpClient::new(config)`, no executor | Isolates which layer drops the headers |

Scenario 3 is what makes the suite diagnostic rather than merely red. Its
*presence* assertions pass today, proving the headers are built and applied
correctly when no executor is injected — so scenario 2's remaining failure
points at the executor's header plumbing rather than at the header definitions.
Its *value* assertion for `X-Fern-SDK-Version` still fails, on an unrelated
defect (see the failure table below).

Scenario 2 must be a *faithful* replica for any of that to hold. It reconstructs
`CliExecutor::new`'s arguments by hand, since the real bridge reads them off a
live `AppContext`; every hand-reconstructed argument is a place the replica can
lie. That is not hypothetical — passing `Vec::new()` for `global_headers` is
exactly what produced the retracted ENG-10092 finding. The replica therefore
enumerates its known divergences (global headers, auth provider, base URL)
instead of carrying a general "keep in sync" note.

Scenario 1 asserts the `User-Agent` only. Whether the OpenAPI path — which
carries today's real command traffic — *should* also send the identity headers
is a genuine open question, but it is a product decision about the CLI surface
rather than a regression, so it is recorded for the upstream ticket rather than
asserted here. Adding it later is one more assertion in an existing scenario.

Everything required is public: `HttpConfig::new` / `build_client`
(`src/http.rs:127,333`), `NoAuthProvider` (`src/auth/provider.rs:152`),
`CliExecutor::new` (`src/sdk_executor.rs:79`), and both `HttpClient`
constructors. Integration tests link the lib as `fern_cli_sdk`, and `hedra_sdk`
is a direct dependency of the root package, so both are importable.

### Assertions compare against sources of truth, never frozen literals

This is the load-bearing decision. The entire bug class here is *a constant
drifting away from the manifest that should govern it*. A test asserting the
literal `"1.0.0-dev"` would need editing at every release and would never catch
the drift it exists to catch.

| Header | Compared against |
|---|---|
| `User-Agent` | shape `hedra-cli/{env!("CARGO_PKG_VERSION")}` — the product token is derived from `HttpConfig`'s name (`"hedra"` → `hedra-cli`, per `src/user_agent.rs`), not hardcoded |
| `X-Hedra-Spec-Version` (SDK-direct path) | `info.version` parsed from the embedded `cli/hedra/openapi0.json` |
| `X-Hedra-Spec-Version` (executor path) | the `default` on the matching `x-fern-global-headers` entry in that same file — a *different field*, because a different mechanism supplies the value on this path (see the correction above). Both read `3.2.2` today, and nothing enforces that |
| `X-Fern-SDK-Version` | `env!("CARGO_PKG_VERSION")` |
| `X-Fern-SDK-Name`, `X-Fern-Language` | exact equality against `hedra_sdk` / `Rust` — these are generator-fixed identity constants, not versions, so there is no drift to tolerate and a weaker "present and non-empty" check would accept a truncated or garbled value |

In a `tests/` integration test `env!("CARGO_PKG_VERSION")` resolves to the root
`hedra-cli` package version, so the `X-Fern-SDK-Version` assertion transitively
guards #52's cross-manifest alignment invariant as well.

`User-Agent` additionally covers the three behaviors `src/http.rs:198-260`
documents: the suffix is appended when set, a blank suffix is ignored, and a
header-invalid suffix is ignored. These use the
`with_user_agent_suffix_override` builder (`src/http.rs:178`) rather than the
`HEDRA_USER_AGENT_SUFFIX` env var, so the cases stay parallel-safe. Env-var
variants would need `serial_test`, which is exactly the flakiness `src/pager.rs`
carries a hand-fix for.

### Expected result: two failures, two distinct root causes

| Failing assertion | Root cause |
|---|---|
| Scenario 2: `X-Fern-*` absent | the CLI's executor never picks up the trio that `with_executor` makes it responsible for — `ClientConfig::custom_headers` is never merged into `CliExecutor::global_headers` |
| Scenario 3: `X-Fern-SDK-Version` is `0.1.0`, not `1.0.0-dev` | hardcoded literal, no channel from `fern generate --version` |

~~| Scenario 2: `X-Hedra-Spec-Version` absent | same bypass, request-level
`additional_headers` |~~

**Retracted — this third row was never a real failure.** It came from the
harness passing `Vec::new()` as the executor's `global_headers`. The real CLI
supplies that header on its own channel, and both the corrected harness and a
trace of the built binary confirm it arrives. See the correction in *Problem*.
Its assertion stays in the suite, inverted: as a **passing** regression lock —
the only one in the repo pinning `X-Hedra-Spec-Version` to the wire.

The two remaining failures are independent: one is a header-plumbing gap in the
generated bridge, the other a stale constant that survives any fix to the first.

The retraction is also the reason the suite keeps a passing control
(`executor_path_still_sends_user_agent`) whose stated purpose is narrow. It
shows client-level headers survive the executor, so a missing header is
channel-specific rather than a wholesale wipe. It does *not* prove the request
was sent — `capture_one` already asserts exactly one request arrived, so a
scenario that never fired fails before any header is read.

### Survival against regeneration

`tests/` does not currently exist, and the generator does not emit it. Per the
`.gitattributes` precedent in `.fernignore` — where dropping the entry caused
PR #29 to delete the file outright — a file that exists but is not emitted is
deleted by regeneration unless `.fernignore` lists it.

**Adding `tests/` to `.fernignore` is a required part of this change**, not an
optional hardening step.

## Verification

Verification runs in a `rust:latest` container, matching the practice
established by #40 — it tracks CI's `ubuntu-latest` + unpinned toolchain more
closely than the macOS host. A local `cargo` (1.97.1) is available for a quick
inner loop, but the container run is the authoritative one:

```
cargo test --locked --test wire_contract
```

Expected: **7 passed, 2 failed** — scenario 1 whole, scenario 3's non-version
assertions, and scenario 2's `X-Hedra-Spec-Version` and `User-Agent` assertions
pass; the two failures in the table above are the only ones.

"It compiles and fails for the stated reasons" is the acceptance bar — a failure
for any *other* reason means the harness is wrong, not the code. That bar cuts
both ways, and it is worth stating why: an *expected* failure was accepted here
without asking whether the harness could manufacture it, and one could. When a
red assertion is the deliverable, confirm the replica reproduces the real path
before reading its failure as evidence about the code.

## Deliverables

1. `tests/wire_contract.rs`
2. `.fernignore` entry for `tests/`, with a comment recording why
3. Draft PR whose red CI is the point, with the two failures and their causes
   explained in the body
4. A fern-config / fern-cli-generator ticket for the durable fix, citing the
   failing test names as acceptance criteria, and noting that `CliExecutor`
   already carries a `global_headers` channel (`src/sdk_executor.rs:64`,
   populated at `src/openapi/app.rs:2292`) — so the fix is plumbing the SDK's
   `custom_headers` into it, not a generator rewrite.

   That lead is now more than a hunch: `X-Hedra-Spec-Version` already rides that
   exact channel to the wire, so the fix is extending a proven mechanism to
   three more headers. **The ticket must not claim an ENG-10092 regression** —
   it is not regressed, and the harness now locks it.

## Risks

- ~~**Cannot compile locally.** The API surface was verified by reading public
  signatures, not by building. First container run may need import fixes.~~
  Retired: this rested on a false premise (a local toolchain does exist), and
  the risk did not materialise anyway — all three scenarios compiled on the
  first attempt with no signature or import adjustments.
- **Red CI on a draft PR** is deliberate and must be stated in the PR body, or
  a reviewer will read it as broken work.
- **Scenario 2 replicates `cli/hedra/sdk.rs` rather than calling it**, because
  that file belongs to the binary, not the lib, and is unreachable from an
  integration test. ~~If the generated bridge changes shape, the replica can
  drift from it. Mitigated by a comment pinning the replica to its source.~~

  **Materialised, and the mitigation was the wrong one.** The risk was framed as
  *future* drift from *regeneration*, so the mitigation was a pointer to the
  source file. The divergence that actually occurred was present from the first
  commit and was a hand-simplification: `Vec::new()` for `global_headers`, which
  the plan prescribed literally. A comment saying "keep this in sync" cannot
  catch that — the replica looked exactly like its source, minus one argument
  nobody had a reason to question.

  Re-mitigated: the replica now enumerates its three known divergences from the
  real path (global headers — with a note not to re-simplify it — auth provider,
  and base URL) rather than pointing at the file and hoping. A hand-written
  replica of a generated file should be assumed unfaithful until each
  reconstructed argument is checked against the original.

- **A red assertion can be manufactured by the harness.** A suite whose
  deliverable is failure has no signal distinguishing "the code is broken" from
  "the test is wrong" — both are red, and the expected-failure list makes the
  second look like success. This is what happened here. Partially mitigated by
  scenario 3's passing controls; fully only by an out-of-band oracle, which is
  what the binary trace provided.

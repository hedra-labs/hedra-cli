# Wire-contract harness for hedra-cli outbound headers

**Date:** 2026-08-10
**Status:** Approved design
**Baseline:** `ed9886b` (main), all three crates at `1.0.0-dev`

## Problem

Nothing in this repository proves what a server actually receives from the CLI.
Three identity headers are declared in generated code, and static reading of the
call graph says at least two of them never reach the wire:

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

So on the CLI's SDK path, all four headers are constructed and then discarded.
This defeats ENG-10092, which added `X-Hedra-Spec-Version` to the CLI surface
and was verified by grepping the *generated tree* — a check that cannot
distinguish "emitted" from "delivered".

Separately, `X-Fern-SDK-Version` is a hardcoded `"0.1.0"` literal untouched
since the first commit. It survived `0.2.0 → 0.9.9 → 1.0.0-dev` and every
regeneration; `fern generate --version` has no channel to it. After #52
(ENG-10219) aligned all three manifests to `1.0.0-dev`, this constant is the
last place in the repo still claiming `0.1.0`.

Impact today is latent: `cli/hedra/custom.rs` is a 41-line template with every
example commented out, so nothing constructs the SDK client at runtime. Real
command traffic goes through `src/openapi/*` on `src/http.rs`'s own reqwest
client, which sends only `User-Agent`. The gap bites whoever writes the first
custom command.

## Goals

Build an executable specification for the outbound wire contract that:

1. States what a server *should* receive, for each of the three ways a request
   can leave this CLI.
2. Fails today, precisely, once per distinct root cause — so the fix ticket
   cannot close half the problem and appear done.
3. Detects future drift automatically, without needing an edit at each release.

## Non-goals

- **Fixing the bypass.** Every file involved (`hedra-sdk/**`, `src/http.rs`,
  `cli/hedra/sdk.rs`) is generator-owned — absent from `.fernignore` — so an
  in-repo patch is reverted by the next regeneration. The durable fix belongs
  upstream in fern-config / fern-cli-generator and is filed separately.
- Asserting auth headers, retries, or TLS behavior.
- Driving the real compiled binary. Only the OpenAPI path is reachable that
  way, so it could not observe the headers this work is about.

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
correctly when no executor is injected — so scenario 2's failure points
unambiguously at `with_executor` rather than at the header definitions. Its
*value* assertion for `X-Fern-SDK-Version` still fails, on an unrelated defect
(see the failure table below).

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
| `X-Hedra-Spec-Version` | `info.version` parsed from the embedded `cli/hedra/openapi0.json` |
| `X-Fern-SDK-Version` | `env!("CARGO_PKG_VERSION")` |
| `X-Fern-SDK-Name`, `X-Fern-Language` | present and non-empty |

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

### Expected result: three failures, three distinct root causes

| Failing assertion | Root cause |
|---|---|
| Scenario 2: `X-Fern-*` absent | `with_executor` skips `apply_custom_headers` (config-level headers) |
| Scenario 2: `X-Hedra-Spec-Version` absent | same bypass, request-level `additional_headers` |
| Scenario 3: `X-Fern-SDK-Version` is `0.1.0`, not `1.0.0-dev` | hardcoded literal, no channel from `fern generate --version` |

The first two share a mechanism but differ in which header channel is lost, and
a partial fix could plausibly restore one and not the other — so they are
asserted separately. The third is an independent defect that survives any fix
to the bypass.

### Survival against regeneration

`tests/` does not currently exist, and the generator does not emit it. Per the
`.gitattributes` precedent in `.fernignore` — where dropping the entry caused
PR #29 to delete the file outright — a file that exists but is not emitted is
deleted by regeneration unless `.fernignore` lists it.

**Adding `tests/` to `.fernignore` is a required part of this change**, not an
optional hardening step.

## Verification

No local Rust toolchain, so verification runs in a `rust:latest` container,
matching the practice established by #40:

```
cargo test --locked --test wire_contract
```

Expected: the suite compiles, scenario 1 and scenario 3's non-version
assertions pass, and exactly three assertions fail with the messages above.
"It compiles and fails for the stated reasons" is the acceptance bar — a
failure for any *other* reason means the harness is wrong, not the code.

## Deliverables

1. `tests/wire_contract.rs`
2. `.fernignore` entry for `tests/`, with a comment recording why
3. Draft PR whose red CI is the point, with the three failures and their causes
   explained in the body
4. A fern-config / fern-cli-generator ticket for the durable fix, citing the
   failing test names as acceptance criteria, and noting that `CliExecutor`
   already carries a `global_headers` channel (`src/sdk_executor.rs:64`,
   populated at `src/openapi/app.rs:2292`) — so the fix is plumbing the SDK's
   `custom_headers` into it, not a generator rewrite

## Risks

- **Cannot compile locally.** The API surface was verified by reading public
  signatures, not by building. First container run may need import fixes.
- **Red CI on a draft PR** is deliberate and must be stated in the PR body, or
  a reviewer will read it as broken work.
- **Scenario 2 replicates `cli/hedra/sdk.rs` rather than calling it**, because
  that file belongs to the binary, not the lib, and is unreachable from an
  integration test. If the generated bridge changes shape, the replica can
  drift from it. Mitigated by a comment pinning the replica to its source.

# Hedra CLI

Command-line interface for the [Hedra](https://hedra.com) **API v3** — browse the
model catalog, submit generation jobs, follow their progress, and manage keys,
webhooks, log drains and billing from your terminal.

The binary is `hedra-cli`. Every API resource is a subcommand (e.g. `hedra-cli jobs submit`).

## Table of contents

- [Installation](#installation)
- [Authentication](#authentication)
- [Quick start](#quick-start)
- [Commands](#commands)
  - [`jobs`](#jobs)
  - [`models`](#models)
  - [`files`](#files)
  - [`keys`](#keys)
  - [`tokens`](#tokens)
  - [`billing`](#billing)
  - [`webhooks`](#webhooks)
  - [`log-drains`](#log-drains)
  - [`workspaces`](#workspaces)
  - [Built-in commands](#built-in-commands)
- [Global flags](#global-flags)
- [Output formats](#output-formats)
- [Pagination](#pagination)
- [Environment variables](#environment-variables)
- [Shell completion](#shell-completion)

## Installation

### npm

```bash
npm install --global @hedra/cli
```

### Homebrew

```bash
brew install hedra-labs/tap/hedra-cli
```

### Installer script

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/hedra-labs/hedra-cli/releases/latest/download/hedra-cli-installer.sh | sh
```

### Build from source

Install the [Rust toolchain](https://rustup.rs/), then:

```bash
git clone https://github.com/hedra-labs/hedra-cli.git
cd hedra-cli
cargo build --release
./target/release/hedra-cli --help
```

Copy `target/release/hedra-cli` somewhere on your `PATH` (e.g. `~/.local/bin`).

Prebuilt archives and installers are published from tagged releases.

## Authentication

Requests authenticate with `Authorization: Bearer <credential>`, where the credential is
either an API key (`<key_id>:<secret>`) or an ephemeral token minted via `tokens create`.
There are two ways to get one:

```bash
# 1. Browser login (recommended). Opens AuthKit, then exchanges the resulting
#    session for a durable API key stored in your OS keyring.
hedra-cli auth login

# 2. Paste an existing key from the Hedra console straight into the keyring.
hedra-cli auth login --with-token
```

`auth login` discovers the authorization server from the API at runtime (RFC 9728/8414)
rather than hard-coding it, then renews the key it already holds when possible and mints
a new one otherwise. Add `--no-browser` to print the URL instead of opening it.

Alternatively, set the key in the environment — it takes precedence over the keyring:

```bash
export HEDRA_API_KEY="<key_id>:<secret>"
```

Check every credential source the CLI can see (and spot shadowing) with:

```bash
hedra-cli auth status
```

`hedra-cli auth logout` clears the stored credentials. The model catalog
(`hedra-cli models …`) and the webhook public key are readable without credentials.

## Quick start

```bash
hedra-cli auth login

hedra-cli models list                          # the model catalog
hedra-cli models get --model veo-31             # input schema for one model

# Submit a job and follow it
hedra-cli jobs submit --model veo-31 --input '{"prompt": "a fox sprinting across fresh snow", "aspect_ratio": "16:9"}'
hedra-cli jobs get-status --job-id <JOB_ID>
hedra-cli jobs get --job-id <JOB_ID>
```

The CLI talks to production (`https://api.hedra.com/v3`) by default. Point it at staging
with `HEDRA_ENV=staging`, or at an arbitrary host with `--base-url` / `HEDRA_CLI_BASE_URL`.

## Commands

Run `hedra-cli <resource> --help`, or `hedra-cli <resource> <method> --help`, for full
details on any command. The flags below are command-specific; see
[Global flags](#global-flags) for options available everywhere.

### `jobs`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra-cli jobs submit` | Submit a generation job to any model | `POST /models/{model}` |
| `hedra-cli jobs submit-<model>` | Typed variant per model (75 of them) | `POST /models/<model>` |
| `hedra-cli jobs list` | List past jobs | `GET /jobs` |
| `hedra-cli jobs get` | Fetch the result envelope | `GET /jobs/{job_id}` |
| `hedra-cli jobs get-status` | Poll progress | `GET /jobs/{job_id}/status` |
| `hedra-cli jobs stream` | Follow progress over SSE | `GET /jobs/{job_id}/stream` |
| `hedra-cli jobs list-job-logs` | Page through a job's logs | `GET /jobs/{job_id}/logs` |

| Command | Flag | Description |
|---------|------|-------------|
| `submit` | `--model <STRING>` | Model public id (see `models list`) |
| `submit` | `--input <JSON_OBJECT>` | Model input, validated against the model's schema |
| `submit` | `--webhook <STRING>` | URL to receive a signed completion webhook |
| `submit` | `--idempotency-key <STRING>` | Replays the original ack instead of enqueueing a duplicate |
| `list` | `--limit <N>` / `--cursor <VALUE>` | Cursor pagination (default limit `20`) |
| `get` / `get-status` / `stream` / `list-job-logs` | `--job-id <STRING>` | Job to query |
| `get-status` | `--logs-after <VALUE>` | Only logs after this marker |
| `stream` | `--last-event-id <VALUE>` / `--no-stream` | Resume the event stream / disable streaming |
| `list-job-logs` | `--limit <N>` / `--cursor <VALUE>` | Cursor pagination |

```bash
hedra-cli jobs submit --model veo-31 --input '{
  "prompt": "a fox sprinting across fresh snow",
  "aspect_ratio": "16:9"
}'
hedra-cli jobs stream --job-id <JOB_ID>
```

Each model also gets its own `submit-<model>` subcommand that flattens the model's input
schema into typed, validated flags — with enumerated values, so `--help` and shell
completion tell you what is accepted:

```bash
hedra-cli jobs submit-nano-banana-pro \
  --input.prompt "a fox sprinting across fresh snow" \
  --input.aspect-ratio 16:9 \
  --input.resolution 2K
```

### `models`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra-cli models list` | List models | `GET /models` |
| `hedra-cli models get` | Model detail (input schema, routing) | `GET /models/{model}` |
| `hedra-cli models list-voices` | TTS voice catalog for a model | `GET /models/{model}/voices` |
| `hedra-cli models list-model-jobs` | Jobs submitted to one model | `GET /models/{model}/jobs` |
| `hedra-cli models get-openapi` | Per-model OpenAPI spec | `GET /models/{model}/openapi.json` |
| `hedra-cli models estimate` | Cost/ETA without queuing | `POST /models/{model}/estimate` |

| Command | Flag | Description |
|---------|------|-------------|
| `list` | `--modality <VALUE>` | Filter by modality (e.g. `video`, `image`, `audio`) |
| `get` / `list-voices` / `get-openapi` / `estimate` / `list-model-jobs` | `--model <STRING>` | Model id |
| `estimate` | `--input <JSON_OBJECT>` | Input to price |
| `list-model-jobs` | `--limit <N>` / `--cursor <VALUE>` | Cursor pagination |

```bash
hedra-cli models list --modality video
hedra-cli models estimate --model veo-31 --input '{"prompt": "a fox", "aspect_ratio": "16:9"}'
```

### `files`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra-cli files upload` | Upload a file for reference inputs | `POST /files` |

```bash
# Upload, then use the returned URL as image / audio / video input on submit
hedra-cli files upload --file ./headshot.png
```

`--file` also accepts `@PATH`, `-` for stdin, and `\@LITERAL` to pass a literal `@`.

### `keys`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra-cli keys create` | Mint an API key (secret shown once) | `POST /keys` |
| `hedra-cli keys list` | List keys (no secrets) | `GET /keys` |
| `hedra-cli keys rotate` | Rotate a key's secret | `POST /keys/{key_id}/rotate` |
| `hedra-cli keys revoke` | Revoke a key | `DELETE /keys/{key_id}` |

| Command | Flag | Description |
|---------|------|-------------|
| `create` | `--name <STRING>` / `--scopes <VALUE>` / `--kind <STRING>` / `--workspace-id <STRING>` / `--expires-at <STRING>` | Key attributes |
| `rotate` | `--key-id <STRING>` / `--grace-period-seconds <N>` | Target key, and how long the old secret keeps working |
| `revoke` | `--key-id <STRING>` | Target key |

### `tokens`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra-cli tokens create` | Mint an ephemeral browser token | `POST /tokens` |

| Flag | Description |
|------|-------------|
| `--ttl-seconds <N>` | Token lifetime |
| `--scopes <VALUE>` | Subset of the minting key's scopes |

### `billing`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra-cli billing get-balance` | Current credit balance | `GET /balance` |
| `hedra-cli billing get-usage` | Usage over a time range | `GET /usage` |

| Command | Flag | Description |
|---------|------|-------------|
| `get-usage` | `--start <VALUE>` / `--end <VALUE>` | Time range |
| `get-usage` | `--group-by <VALUE>` | Aggregation dimension |

```bash
hedra-cli billing get-usage --start 2026-08-01 --group-by model
```

### `webhooks`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra-cli webhooks get-public-key` | ed25519 public key for signature verification | `GET /webhooks/public-key` |
| `hedra-cli webhooks get-default` | Read the account-wide default webhook | `GET /webhooks/default` |
| `hedra-cli webhooks put-default` | Set the default webhook | `PUT /webhooks/default` |
| `hedra-cli webhooks delete-default` | Remove the default webhook | `DELETE /webhooks/default` |
| `hedra-cli webhooks test-default` | Send a test delivery | `POST /webhooks/default/test` |
| `hedra-cli webhooks list-deliveries` | Delivery history | `GET /webhooks/deliveries` |
| `hedra-cli webhooks redeliver` | Replay a delivery for one job | `POST /webhooks/deliveries/{job_id}/redeliver` |

| Command | Flag | Description |
|---------|------|-------------|
| `put-default` | `--url <STRING>` / `--enabled <BOOLEAN>` | Destination and on/off |
| `list-deliveries` | `--limit <N>` / `--cursor <VALUE>` | Cursor pagination |
| `redeliver` | `--job-id <STRING>` | Job whose delivery to replay |

### `log-drains`

Stream structured request logs to an external sink.

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra-cli log-drains list-log-drains` | List drains | `GET /log-drains` |
| `hedra-cli log-drains create-log-drain` | Create a drain | `POST /log-drains` |
| `hedra-cli log-drains get-log-drain` | Drain detail | `GET /log-drains/{drain_id}` |
| `hedra-cli log-drains update-log-drain` | Update a drain | `PATCH /log-drains/{drain_id}` |
| `hedra-cli log-drains delete-log-drain` | Delete a drain | `DELETE /log-drains/{drain_id}` |
| `hedra-cli log-drains test-log-drain` | Send a test batch | `POST /log-drains/{drain_id}/test` |

| Command | Flag | Description |
|---------|------|-------------|
| `create-log-drain` / `update-log-drain` | `--url <STRING>` / `--name <STRING>` / `--enabled <BOOLEAN>` | Destination, label, on/off |
| `create-log-drain` / `update-log-drain` | `--format-param <STRING>` / `--headers <VALUE>` / `--batch-size <N>` / `--secret <STRING>` | Payload format, extra headers, batching, signing secret |
| `get-log-drain` / `update-log-drain` / `delete-log-drain` / `test-log-drain` | `--drain-id <STRING>` | Target drain |

### `workspaces`

Local credential management across workspaces. The CLI holds one API key per workspace
you have logged into, and `select` switches which one is the active credential.

| Command | Description |
|---------|-------------|
| `hedra-cli workspaces list` | List accessible workspaces. `active` carries `*` on the current credential and `key_held` marks the ones `select` can switch to offline. This is the default — bare `hedra-cli workspaces` runs it |
| `hedra-cli workspaces select --workspace-id <ID>` | Make that workspace's key the active credential, and report the result as an object (`active`, `key_id`, `workspace_id`, `workspace_name`) |

Both emit through the standard output pipeline like every generated command, so
`--format` / `--query` / `--quiet` apply (table on a TTY, JSON when piped). The listing
is what `hedra-cli auth login` prints once the browser round-trip completes, so the
post-login summary and `workspaces list` are the same view.

`select` is offline when a key for the target is already held; otherwise it launches a
browser login hinted at that workspace's organization and makes the freshly minted key
active. Note that `HEDRA_API_KEY`, if set, shadows the keyring — the CLI warns when that
would make the selected workspace not the one actually used.

### Built-in commands

| Command | Description |
|---------|-------------|
| `hedra-cli auth login` / `logout` / `status` | Manage stored credentials (OS keyring) |
| `hedra-cli completion <bash\|zsh\|fish\|powershell>` | Generate a shell completion script |
| `hedra-cli man` | Generate a man page (roff) |
| `hedra-cli generate-skills` | Generate `SKILL.md` files for AI-agent integration |

## Global flags

Available on every command:

| Flag | Description |
|------|-------------|
| `--dry-run` | Validate locally and print the HTTP request without sending it |
| `--json <JSON\|->` | Supply a request body as JSON (`-` reads stdin) |
| `--params <JSON>` | Merge extra parameters as JSON (overrides individual flags) |
| `--format <FORMAT>` | Output format (see below) |
| `--query <EXPR>` | JMESPath expression applied to the response before formatting |
| `--base-url <URL>` | Override the API base URL |
| `--page-all` | Auto-paginate and stream results as NDJSON |
| `--page-limit <N>` | Max pages to fetch when auto-paginating (default `10`) |
| `--page-delay <MS>` | Delay between page fetches (default `100`) |
| `--user-agent-suffix <TOKEN>` | Product token appended to the User-Agent (e.g. `my-app/1.0`) |
| `--schema` | Print a machine-readable JSON schema for this scope |
| `--spec` / `--spec-raw` | Print the effective / byte-exact embedded OpenAPI spec |
| `--spec-version <VALUE>` | Override the `X-Hedra-Spec-Version` header (default `3.3.0`) |
| `--no-extract` | Print the full response body instead of the extracted return value |
| `--no-retry` | Disable this operation's declared retries, including on network errors |
| `--no-pager` | Disable the pager on interactive terminals |
| `--debug` | Dump the HTTP request and response to stderr |
| `-q, --quiet` | Suppress stdout on success (errors still go to stderr) |
| `-h, --help` / `-V, --version` | Help / version |

Not every flag applies to every command — `--json`, `--page-*` and `--no-extract` appear
only where they make sense (request bodies, list endpoints).

## Output formats

`--format` controls rendering: `json`, `table`, `yaml`, `csv`, `raw`, `jsonl`, `http`.
The default is `table` when stdout is a TTY and `json` when piped; override the default
with `HEDRA_CLI_OUTPUT`. `raw` emits the unmodified response bytes; `jsonl` emits NDJSON
(arrays flattened); `http` emits the full response like `curl -i`.

```bash
hedra-cli models list --format table
hedra-cli models list --format json | jq '.data[].id'
```

## Pagination

For list endpoints, `--page-all` follows cursor pages automatically and streams each
result as a line of NDJSON; bound it with `--page-limit` and pace it with `--page-delay`:

```bash
hedra-cli jobs list --page-all --page-limit 5
```

## Environment variables

| Variable | Description |
|----------|-------------|
| `HEDRA_API_KEY` | API key (`<key_id>:<secret>`, sent as `Authorization: Bearer …`). Shadows the keyring |
| `HEDRA_CLI_BASE_URL` | Override the API base URL |
| `HEDRA_ENV` | Target environment: `prod` (default) or `staging`. Selects the API resource base, the login/auth discovery origin, and (for `staging`) derives the API base URL unless overridden |
| `HEDRA_CLI_OUTPUT` | Default output format when `--format` is not given |
| `HEDRA_CLI_USER_AGENT_SUFFIX` | Product token appended to the User-Agent (`--user-agent-suffix` wins) |
| `HEDRA_CLI_CA_BUNDLE` | Path to a PEM file with extra trust roots (or `SSL_CERT_FILE`) |
| `HEDRA_CLI_INSECURE=1` | Skip TLS verification (debugging only) |
| `HEDRA_CLI_PROXY` | HTTP(S) proxy URL |
| `HEDRA_CLI_TIMEOUT_SECS` | Total request timeout in seconds |

A `.env` file in the working directory is loaded on startup. Standard `HTTPS_PROXY` /
`HTTP_PROXY` / `NO_PROXY` / `SSL_CERT_FILE` are also honored.

## Shell completion

```bash
# zsh — add to a directory on your $fpath
hedra-cli completion zsh > ~/.zfunc/_hedra-cli

# bash
hedra-cli completion bash > /etc/bash_completion.d/hedra-cli
```

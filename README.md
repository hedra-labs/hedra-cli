# Hedra CLI

Command-line interface for the [Hedra](https://hedra.com) **API v3** — browse the
model catalog, submit generation jobs to the queue, follow their progress, and manage
API keys from your terminal.

The binary is `hedra`. Every API resource is a subcommand (e.g. `hedra queue submit`).

## Table of contents

- [Installation](#installation)
- [Authentication](#authentication)
- [Quick start](#quick-start)
- [Commands](#commands)
  - [`queue`](#queue)
  - [`requests`](#requests)
  - [`models`](#models)
  - [`files`](#files)
  - [`keys`](#keys)
  - [`tokens`](#tokens)
  - [`webhooks`](#webhooks)
  - [Built-in commands](#built-in-commands)
- [Global flags](#global-flags)
- [Output formats](#output-formats)
- [Pagination](#pagination)
- [Environment variables](#environment-variables)
- [Shell completion](#shell-completion)

## Installation

### Build from source

Install the [Rust toolchain](https://rustup.rs/), then:

```bash
git clone https://github.com/hedra-labs/hedra-cli.git
cd hedra-cli
cargo build --release
./target/release/hedra --help
```

Copy `target/release/hedra` somewhere on your `PATH` (e.g. `~/.local/bin`) to run it as `hedra`.

> Prebuilt installers (shell / PowerShell / Homebrew) are produced by the release
> workflow once a tagged release is published; until then, build from source.

## Authentication

The API authenticates with `Authorization: Bearer <api key>`, where an API key is the
`<key_id>:<secret>` credential from the Hedra console. Provide it in any of these ways
(checked in this order):

```bash
# 1. Environment variable (also auto-loaded from a .env file in the working dir)
export HEDRA_API_KEY="<key_id>:<secret>"

# 2. OS keyring (stored once, reused across runs)
hedra auth login --with-token
```

Check which sources are visible (and detect shadowing) with:

```bash
hedra auth status
```

The model catalog (`hedra models …`) and the webhook public key are readable without
credentials.

## Quick start

```bash
export HEDRA_API_KEY="<key_id>:<secret>"

hedra models list                        # the model catalog
hedra models get --model kling-o3-pro    # schema + variants for one model

# Submit a job and follow it
hedra queue submit --model kling-o3-pro --input '{"prompt": "a fox sprinting across fresh snow"}'
hedra requests get_status --request-id <REQUEST_ID>
hedra requests get --request-id <REQUEST_ID>
```

The CLI talks to production (`https://api.hedra.com/v3`) by default. Override the base
URL with `--base-url` or `HEDRA_BASE_URL` (e.g. for a mock server in tests).

## Commands

Run `hedra <resource> --help`, or `hedra <resource> <method> --help`, for full details on
any command. The flags below are command-specific; see [Global flags](#global-flags) for
options available everywhere.

### `queue`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra queue submit` | Submit a generation job | `POST /queue/{model}` |

| Flag | Description |
|------|-------------|
| `--model <STRING>` | Model family or variant id (e.g. `kling-o3-pro`, `kling-o3-pro-i2v`) |
| `--input <JSON_OBJECT>` | Schema-validated model input |
| `--webhook <STRING>` | Webhook URL to POST the terminal result to |
| `--idempotency-key <STRING>` | Dedupe key for safe retries |
| `--priority <STRING>` | `normal` / `low` |

```bash
hedra queue submit --model kling-o3-pro --input '{
  "prompt": "a fox sprinting across fresh snow",
  "aspect_ratio": "16:9"
}'
```

### `requests`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra requests list` | List past requests | `GET /requests` |
| `hedra requests get` | Fetch the result envelope | `GET /requests/{request_id}` |
| `hedra requests get_status` | Poll progress | `GET /requests/{request_id}/status` |
| `hedra requests stream` | Follow progress over SSE | `GET /requests/{request_id}/stream` |

| Command | Flag | Description |
|---------|------|-------------|
| `list` | `--limit <N>` / `--cursor <VALUE>` | Cursor pagination (default limit `20`) |
| `get` / `get_status` / `stream` | `--request-id <STRING>` | Request to query |
| `stream` | `--last-event-id <VALUE>` | Resume the event stream |

```bash
hedra requests list --limit 5
hedra requests stream --request-id <REQUEST_ID>
```

### `models`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra models list` | List model families | `GET /models` |
| `hedra models get` | Family or variant detail (input schema, routing) | `GET /models/{model}` |
| `hedra models list_voices` | TTS voice catalog for a model | `GET /models/{model}/voices` |
| `hedra models get_openapi` | Per-model OpenAPI spec | `GET /models/{model}/openapi.json` |
| `hedra models estimate` | Cost/ETA without queuing | `POST /models/{model}/estimate` |

| Command | Flag | Description |
|---------|------|-------------|
| `list` | `--type <VALUE>` | Filter by modality (`video`, `image`, `audio`) |
| `get` / `list_voices` / `get_openapi` / `estimate` | `--model <STRING>` | Model id |
| `estimate` | `--input <JSON_OBJECT>` | Input to price |

```bash
hedra models list --type video
hedra models estimate --model kling-o3-pro --input '{"prompt": "a fox"}'
```

### `files`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra files upload` | Upload a file for reference inputs | `POST /files` |

```bash
# Upload, then use the returned URL as image_url / audio_url / video_url on submit
hedra files upload --file ./headshot.png
```

### `keys`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra keys create` | Mint an API key (secret shown once) | `POST /keys` |
| `hedra keys list` | List keys (no secrets) | `GET /keys` |
| `hedra keys rotate` | Rotate a key's secret | `POST /keys/{key_id}/rotate` |
| `hedra keys revoke` | Revoke a key | `DELETE /keys/{key_id}` |

| Command | Flag | Description |
|---------|------|-------------|
| `create` | `--name <STRING>` / `--scopes <VALUE>` / `--kind <personal\|service>` / `--workspace-id <STRING>` / `--expires-at <STRING>` | Key attributes |
| `rotate` / `revoke` | `--key-id <STRING>` | Target key |

### `tokens`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra tokens create` | Mint an ephemeral browser token | `POST /tokens` |

| Flag | Description |
|------|-------------|
| `--ttl-seconds <N>` | Token lifetime |
| `--scopes <VALUE>` | Subset of the minting key's scopes |

### `webhooks`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra webhooks get_public_key` | ed25519 public key for signature verification | `GET /webhooks/public-key` |

### Built-in commands

| Command | Description |
|---------|-------------|
| `hedra auth login` / `logout` / `status` | Manage stored credentials (OS keyring) |
| `hedra completion <bash\|zsh\|fish\|powershell>` | Generate a shell completion script |
| `hedra man` | Generate a man page (roff) |
| `hedra generate-skills` | Generate `SKILL.md` files for AI-agent integration |

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
| `--schema` | Print a machine-readable JSON schema for this scope |
| `--debug` | Dump the HTTP request and response to stderr |
| `-q, --quiet` | Suppress stdout on success (errors still go to stderr) |
| `-h, --help` / `-V, --version` | Help / version |

## Output formats

`--format` controls rendering: `json`, `table`, `yaml`, `csv`, `raw`, `jsonl`, `http`.
The default is `table` when stdout is a TTY and `json` when piped. `raw` emits the
unmodified response bytes; `jsonl` emits NDJSON (arrays flattened); `http` emits the full
response like `curl -i`.

```bash
hedra models list --format table
hedra models list --format json | jq '.data[].id'
```

## Pagination

For list endpoints, `--page-all` follows cursor pages automatically and streams each
result as a line of NDJSON; bound it with `--page-limit`:

```bash
hedra requests list --page-all --page-limit 5
```

## Environment variables

| Variable | Description |
|----------|-------------|
| `HEDRA_API_KEY` | API key (`<key_id>:<secret>`, sent as `Authorization: Bearer …`) |
| `HEDRA_BASE_URL` | Override the API base URL |
| `HEDRA_CA_BUNDLE` | Path to a PEM file with extra trust roots (or `SSL_CERT_FILE`) |
| `HEDRA_INSECURE=1` | Skip TLS verification (debugging only) |
| `HEDRA_PROXY` | HTTP(S) proxy URL |
| `HEDRA_TIMEOUT_SECS` | Total request timeout in seconds |

Standard `HTTPS_PROXY` / `HTTP_PROXY` / `NO_PROXY` / `SSL_CERT_FILE` are also honored.

## Shell completion

```bash
# zsh — add to a directory on your $fpath
hedra completion zsh > ~/.zfunc/_hedra

# bash
hedra completion bash > /etc/bash_completion.d/hedra
```

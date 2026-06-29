# Hedra CLI

Command-line interface for the [Hedra](https://hedra.com) Web API — list models and
voices, manage assets, and run generations from your terminal.

The binary is `hedra`. Every API resource is a subcommand (e.g. `hedra voices list_voices`).

## Table of contents

- [Installation](#installation)
- [Authentication](#authentication)
- [Quick start](#quick-start)
- [Commands](#commands)
  - [`voices`](#voices)
  - [`models`](#models)
  - [`assets`](#assets)
  - [`generations`](#generations)
  - [`billing`](#billing)
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

The API authenticates with an API key sent as the `X-API-Key` header. Provide it in any
of these ways (checked in this order):

```bash
# 1. Per-invocation flag
hedra voices list_voices --api-key "$YOUR_KEY"

# 2. Environment variable (also auto-loaded from a .env file in the working dir)
export HEDRA_API_KEY="sk_hedra_..."

# 3. OS keyring (stored once, reused across runs)
hedra auth login
```

Check which sources are visible (and detect shadowing) with:

```bash
hedra auth status
```

## Quick start

```bash
export HEDRA_API_KEY="sk_hedra_..."

hedra billing get_credits          # how many credits you have
hedra models list_models           # available generation models
hedra voices list_voices           # available voices
```

The CLI talks to production (`https://api.hedra.com/web-app/public`) by default. Override
the base URL with `--base-url` or `HEDRA_BASE_URL` (e.g. for staging:
`--base-url https://api.staging.hedra.com/web-app/public`).

## Commands

Run `hedra <resource> --help`, or `hedra <resource> <method> --help`, for full details on
any command. The flags below are command-specific; see [Global flags](#global-flags) for
options available everywhere.

### `voices`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra voices list_voices` | List available voices | `GET /voices` |

```bash
hedra voices list_voices
hedra voices list_voices --query "[?asset.labels[?value=='english']].asset.external_id"
```

### `models`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra models list_models` | List available models | `GET /models` |

| Flag | Description |
|------|-------------|
| `--types <VALUE>` | Filter by model type |

```bash
hedra models list_models
hedra models list_models --types image
```

### `assets`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra assets list_assets` | List assets | `GET /assets` |
| `hedra assets create_asset` | Create an asset | `POST /assets` |
| `hedra assets upload_asset` | Upload bytes to an asset | `POST /assets/{id}/upload` |

| Command | Flag | Description |
|---------|------|-------------|
| `list_assets` | `--type <VALUE>` | Asset type (`text`, `image`, `audio`, `video`, `voice`) |
| `list_assets` | `--ids <VALUE>` | Filter by asset IDs |
| `create_asset` | `--name <STRING>` | Asset name (defaults to the file name) |
| `create_asset` | `--type <STRING>` | Asset type (`text`, `image`, `audio`, `video`, `voice`) |
| `upload_asset` | `--id <UUID>` | Target asset ID |
| `upload_asset` | `--file <PATH \| @PATH \| ->` | File to upload (`-` reads stdin) |

```bash
# Create an image asset, then upload bytes to it
hedra assets create_asset --name "headshot" --type image
hedra assets upload_asset --id <ASSET_UUID> --file ./headshot.png

# List image assets
hedra assets list_assets --type image
```

### `generations`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra generations list_generations` | List generations | `GET /generations` |
| `hedra generations generate_asset` | Start a generation | `POST /generations` |
| `hedra generations get_status` | Get generation status | `GET /generations/{generation_id}/status` |

| Command | Flag | Description |
|---------|------|-------------|
| `list_generations` | `--type <VALUE>` | Filter by generation type |
| `list_generations` | `--created-after <VALUE>` / `--created-before <VALUE>` | Time-range filters |
| `list_generations` | `--agent-thread-id <VALUE>` | Filter by agent thread |
| `list_generations` | `--ids <VALUE>` | Filter by generation IDs |
| `list_generations` | `--limit <N>` / `--offset <N>` | Pagination (default `100` / `0`) |
| `generate_asset` | `--json <JSON \| ->` | Request body as JSON (`-` reads stdin) |
| `get_status` | `--generation-id <UUID>` | Generation to query |

```bash
# Recent generations
hedra generations list_generations --limit 5

# Start a generation from a JSON body
hedra generations generate_asset --json '{
  "type": "video",
  "ai_model_id": "<MODEL_ID>",
  "start_keyframe_id": "<ASSET_ID>",
  "audio_id": "<ASSET_ID>"
}'

# Poll its status
hedra generations get_status --generation-id <GENERATION_UUID>
```

### `billing`

| Command | Description | Endpoint |
|---------|-------------|----------|
| `hedra billing get_credits` | Get credit balance | `GET /billing/credits` |

```bash
hedra billing get_credits
```

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
| `--output <PATH>` | Write binary responses to a file |
| `--base-url <URL>` | Override the API base URL |
| `--page-all` | Auto-paginate and stream results as NDJSON |
| `--page-limit <N>` | Max pages to fetch when auto-paginating (default `10`) |
| `--debug` | Dump the HTTP request and response to stderr |
| `-q, --quiet` | Suppress stdout on success (errors still go to stderr) |
| `-h, --help` / `-V, --version` | Help / version |

## Output formats

`--format` controls rendering: `json`, `table`, `yaml`, `csv`, `raw`, `jsonl`, `http`.
The default is `table` when stdout is a TTY and `json` when piped. `raw` emits the
unmodified response bytes; `jsonl` emits NDJSON (arrays flattened); `http` emits the full
response like `curl -i`.

```bash
hedra voices list_voices --format table
hedra models list_models --format json | jq '.[].id'
```

## Pagination

For list endpoints, `--page-all` follows pages automatically and streams each result as a
line of NDJSON; bound it with `--page-limit`:

```bash
hedra generations list_generations --page-all --page-limit 5
```

## Environment variables

| Variable | Description |
|----------|-------------|
| `HEDRA_API_KEY` | API key (the `X-API-Key` credential) |
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

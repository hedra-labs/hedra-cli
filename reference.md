# Hedra API v3 CLI Reference

Full command reference for `hedra`.

## Commands

- [`hedra files`](#hedra-files)
- [`hedra keys`](#hedra-keys)
- [`hedra models`](#hedra-models)
- [`hedra queue`](#hedra-queue)
- [`hedra requests`](#hedra-requests)
- [`hedra tokens`](#hedra-tokens)
- [`hedra webhooks`](#hedra-webhooks)

---

### `hedra files`

#### `hedra files upload`

Upload File

`POST /files`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

---

### `hedra keys`

#### `hedra keys create`

Create Key

`POST /keys`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra keys list`

List Keys

`GET /keys`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--workspace-id` | `string` | No |  |

#### `hedra keys revoke`

Revoke Key

`DELETE /keys/{key_id}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--key-id` | `string` | Yes |  |

#### `hedra keys rotate`

Rotate Key

`POST /keys/{key_id}/rotate`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--key-id` | `string` | Yes |  |
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

---

### `hedra models`

#### `hedra models estimate`

Estimate

`POST /models/{model}/estimate`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes |  |
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra models get`

Get Model

`GET /models/{model}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes |  |

#### `hedra models get-openapi`

A standalone one-operation OpenAPI spec for this model's submit call.

`GET /models/{model}/openapi.json`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes |  |

#### `hedra models list`

List Models

`GET /models`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--type` | `string` | No |  |

#### `hedra models list-voices`

Voices this model accepts — scoped to the model's voice provider.

`GET /models/{model}/voices`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes |  |

---

### `hedra queue`

#### `hedra queue submit`

Submit

`POST /queue/{model}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--model` | `string` | Yes |  |
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

---

### `hedra requests`

#### `hedra requests get`

Get Request

`GET /requests/{request_id}`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--request-id` | `string` | Yes |  |

#### `hedra requests get-status`

Get Request Status

`GET /requests/{request_id}/status`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--request-id` | `string` | Yes |  |

#### `hedra requests list`

List Requests

`GET /requests`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--limit` | `integer` | No |  |
| `--cursor` | `string` | No |  |

#### `hedra requests stream`

Stream Request

`GET /requests/{request_id}/stream`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--request-id` | `string` | Yes |  |
| `--last-event-id` | `string` | No |  |

---

### `hedra tokens`

#### `hedra tokens create`

Create Token

`POST /tokens`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

---

### `hedra webhooks`

#### `hedra webhooks get-public-key`

Public Key

`GET /webhooks/public-key`

---

## Global flags

These flags are available on every command:

| Flag | Description |
|------|-------------|
| `--dry-run` | Print the HTTP request without sending it |
| `--json <JSON\|->` | Supply the request body as JSON (or `-` for stdin) |
| `--params <JSON>` | Merge extra parameters as JSON |
| `--format <json\|table\|yaml\|csv>` | Output format (default: `json`) |
| `--output <PATH>` | Write binary responses to a file |
| `--base-url <URL>` | Override the API base URL |
| `--page-all` | Auto-paginate and stream all results |
| `--page-limit <N>` | Max pages to fetch (default: `10`) |
| `-q, --quiet` | Suppress stdout on success |
| `-h, --help` | Print help |
| `-V, --version` | Print version |


# Hedra Web API CLI Reference

Full command reference for `hedra`.

## Commands

- [`hedra `](#hedra-)

---

### `hedra `

#### `hedra  create-asset`

Create Asset

`POST /assets`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra  generate-asset`

Generate Asset

`POST /generations`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

#### `hedra  get-credits`

Get Credits

`GET /billing/credits`

#### `hedra  get-status`

Get Status

`GET /generations/{generation_id}/status`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--generation-id` | `string (uuid)` | Yes |  |

#### `hedra  list-assets`

List Assets

`GET /assets`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--type` | `AssetType` | Yes |  |
| `--ids` | `string` | No |  |

#### `hedra  list-generations`

List 

`GET /generations`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--type` | `string` | No |  |
| `--created-before` | `string` | No |  |
| `--created-after` | `string` | No |  |
| `--prompt-query` | `string` | No |  |
| `--agent-thread-id` | `string` | No |  |
| `--ids` | `string` | No |  |
| `--limit` | `integer` | No | Number of items returned in the page. |
| `--offset` | `integer` | No | Number of records skipped. |

#### `hedra  list-models`

List Models

`GET /models`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--types` | `string` | No |  |

#### `hedra  list-voices`

List Voices

`GET /voices`

#### `hedra  upload-asset`

Upload Asset

`POST /assets/{id}/upload`

| Flag | Type | Required | Description |
|------|------|----------|-------------|
| `--id` | `string (uuid)` | Yes |  |
| `--json` | `JSON` | Yes | Request body as JSON (or use individual body-field flags) |

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


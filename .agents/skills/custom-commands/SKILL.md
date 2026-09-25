---
name: hedra-cli-custom-commands
description: How to author custom commands for the hedra-cli CLI using the co-generated SDK.
---

# Custom Commands for `hedra-cli`

## Overview

The `hedra-cli` CLI supports user-authored custom commands that are
compiled into the binary alongside the auto-generated API commands.
Custom commands get a fully-wired SDK client that inherits the CLI's
auth, retries, TLS, base URL, and global headers — zero configuration required.

## Architecture

```
cli/hedra-cli/custom.rs    ← Your command handlers (protected by .fernignore)
cli/hedra-cli/sdk.rs       ← Generated bridge: client() + block_on()
cli/hedra-cli/main.rs      ← Generated entrypoint (calls custom::register)
hedra-cli-sdk/             ← Co-generated typed SDK crate
hedra-cli-types/           ← Co-generated typed model crate
```

## Adding a Custom Command

### 1. Edit `cli/hedra-cli/custom.rs`

This file is protected by `.fernignore` — `fern generate` will never
overwrite it. Register commands in the `register()` function:

```rust
use hedra_cli_sdk::api::*;

pub fn register(app: CliApp) -> CliApp {
    let app = app.command(
        clap::Command::new("jobs-get")
            .about("Get Job")
            .arg(clap::Arg::new("job_id").required(true))
        ,
        |matches, ctx| {
            let job_id = matches.get_one::<String>("job_id").unwrap();
            let client = super::sdk::client(ctx);
            let result = super::sdk::block_on(
                client.jobs.jobs_get(job_id),
            )?;
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            Ok(())
        },
    );
    app
}
```

Then build and test:
```bash
cargo build
hedra-cli jobs-get <job_id>
```

### 2. Available SDK Clients

The `super::sdk::client(ctx)` call returns a `hedra_cli_sdk::api::Client`
with the following sub-clients:

| Field | Type | Description |
|-------|------|-------------|
| `client.jobs` | `hedra_cli_sdk::api::JobsClient` | jobs operations |
| `client.models` | `hedra_cli_sdk::api::ModelsClient` | models operations |
| `client.keys` | `hedra_cli_sdk::api::KeysClient` | keys operations |
| `client.tokens` | `hedra_cli_sdk::api::TokensClient` | tokens operations |
| `client.files` | `hedra_cli_sdk::api::FilesClient` | files operations |
| `client.billing` | `hedra_cli_sdk::api::BillingClient` | billing operations |
| `client.webhooks` | `hedra_cli_sdk::api::WebhooksClient` | webhooks operations |
| `client.log_drains` | `hedra_cli_sdk::api::LogDrainsClient` | log_drains operations |
| `client.chat` | `hedra_cli_sdk::api::ChatClient` | chat operations |

### 3. Key Patterns

**Get the SDK client** (execution-sharing, fully authenticated):
```rust
let client = super::sdk::client(ctx);
```

**Run an async SDK call from a sync handler:**
```rust
let result = super::sdk::block_on(
    client.some_resource.some_method(args),
)?;
```

**Use typed models for request/response serialization:**
```rust
use hedra_cli_sdk::api::*;
```

### 4. Authentication

Custom commands automatically inherit the CLI's authentication.
The following auth schemes are configured:

- **KeyAuth** (bearer): env `HEDRA_API_KEY`

No manual auth wiring is needed in custom command handlers.

## Regeneration Safety

| File | Regenerated? | Notes |
|------|-------------|-------|
| `cli/hedra-cli/custom.rs` | **No** | Protected by `.fernignore` |
| `cli/hedra-cli/sdk.rs` | Yes | Bridges AppContext → SDK client |
| `cli/hedra-cli/main.rs` | Yes | Calls `custom::register(app)` |
| `hedra-cli-sdk/` | Yes | Co-generated typed SDK crate |
| `hedra-cli-types/` | Yes | Co-generated typed models |

After running `fern generate`, your `custom.rs` is preserved. All
generated code (SDK, types, glue, main.rs) is updated to match the
latest API spec. If the SDK surface changes (renamed methods, new
sub-clients), update your `custom.rs` to match.

## Build & Test

```bash
# Build the CLI (includes custom commands)
cargo build

# Run your custom command
hedra-cli <your-command> [args]

# Run with verbose output for debugging
RUST_LOG=debug hedra-cli <your-command> [args]
```

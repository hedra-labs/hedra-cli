---
name: hedra-custom-commands
description: How to author custom commands for the hedra CLI using the co-generated SDK.
---

# Custom Commands for `hedra`

## Overview

The `hedra` CLI supports user-authored custom commands that are
compiled into the binary alongside the auto-generated API commands.
Custom commands get a fully-wired SDK client that inherits the CLI's
auth, retries, TLS, base URL, and global headers — zero configuration required.

## Architecture

```
cli/hedra/custom.rs    ← Your command handlers (protected by .fernignore)
cli/hedra/sdk.rs       ← Generated bridge: client() + block_on()
cli/hedra/main.rs      ← Generated entrypoint (calls custom::register)
hedra-sdk/             ← Co-generated typed SDK crate
hedra-types/           ← Co-generated typed model crate
```

## Adding a Custom Command

### 1. Edit `cli/hedra/custom.rs`

This file is protected by `.fernignore` — `fern generate` will never
overwrite it. Register commands in the `register()` function:

```rust
use hedra_sdk::api::*;

pub fn register(app: CliApp) -> CliApp {
    let app = app.command(
        clap::Command::new("my-command")
            .about("Description of your command")
            .arg(clap::Arg::new("id").required(true)),
        |matches, ctx| {
            let id = matches.get_one::<String>("id").unwrap();
            let client = super::sdk::client(ctx);
            let result = super::sdk::block_on(
                client.resource.get(id),
            )?;
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
            Ok(())
        },
    );
    app
}
```

### 2. Available SDK Clients

The `super::sdk::client(ctx)` call returns a `hedra_sdk::api::Client`
with the following sub-clients:

(Sub-clients are derived from the API spec at generation time.)

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
use hedra_sdk::api::*;
```

### 4. Authentication

Custom commands automatically inherit the CLI's authentication.
The following auth schemes are configured:

- **APIKeyHeader** (header): env `HEDRA_API_KEY`

No manual auth wiring is needed in custom command handlers.

## Regeneration Safety

| File | Regenerated? | Notes |
|------|-------------|-------|
| `cli/hedra/custom.rs` | **No** | Protected by `.fernignore` |
| `cli/hedra/sdk.rs` | Yes | Bridges AppContext → SDK client |
| `cli/hedra/main.rs` | Yes | Calls `custom::register(app)` |
| `hedra-sdk/` | Yes | Co-generated typed SDK crate |
| `hedra-types/` | Yes | Co-generated typed models |

After running `fern generate`, your `custom.rs` is preserved. All
generated code (SDK, types, glue, main.rs) is updated to match the
latest API spec. If the SDK surface changes (renamed methods, new
sub-clients), update your `custom.rs` to match.

## Build & Test

```bash
# Build the CLI (includes custom commands)
cargo build

# Run your custom command
hedra <your-command> [args]

# Run with verbose output for debugging
RUST_LOG=debug hedra <your-command> [args]
```

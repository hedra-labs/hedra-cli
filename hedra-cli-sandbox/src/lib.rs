//! Hand-written support for hedra-cli's sandbox stream verbs (`exec`,
//! `shell`, `logs`, `attach`):
//!
//! - `session` and `stream`: the gRPC client that reaches a sandbox's
//!   sessions through the broker's `SessionService`, with its TLS and proxy
//!   transport, the open's retries and the output spool;
//! - `backoff`: the doubling wait every retry loop, the CLI's own included,
//!   takes;
//! - `console` and `console_text`: raw-mode input and output on a Windows
//!   console, and the text conversions behind them;
//! - behind the `fake-broker` feature, for tests only: `fake_broker`, a
//!   local broker, and `console_host`, a fresh console for the Windows
//!   console tests.
//!
//! `fern generate` never rewrites or deletes this crate, since `.fernignore`
//! lists it. The root `Cargo.toml` names it as a path dependency and has no
//! `[workspace]` table, so `cargo test` at the repository root does not run
//! this crate's tests; run them with
//! `cargo test --locked -p hedra_cli_sandbox`.

pub use crossterm;
pub use prost;
pub use prost_types;
pub use tonic;

/// The messages and the `SessionService` client generated from the vendored
/// protos.
#[allow(clippy::all, missing_docs)]
pub mod pb {
    pub mod hedra {
        pub mod sandbox {
            pub mod v1 {
                tonic::include_proto!("hedra.sandbox.v1");
            }
        }
    }
}

pub mod backoff;
#[cfg(windows)]
pub mod console;
#[cfg(all(windows, feature = "fake-broker"))]
pub mod console_host;
pub mod console_text;
#[cfg(feature = "fake-broker")]
pub mod fake_broker;
pub mod session;
pub mod stream;

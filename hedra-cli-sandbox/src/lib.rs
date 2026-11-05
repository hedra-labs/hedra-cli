//! Hand-written support for hedra-cli's sandbox commands: the gRPC client the
//! stream verbs use to reach a sandbox's sessions through the broker's
//! `SessionService`.
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

#[cfg(windows)]
pub mod console;
#[cfg(all(windows, feature = "fake-broker"))]
pub mod console_host;
pub mod console_text;
#[cfg(feature = "fake-broker")]
pub mod fake_broker;
pub mod session;
pub mod stream;

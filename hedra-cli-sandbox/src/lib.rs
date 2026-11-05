//! Hand-written support for hedra-cli's sandbox commands: the gRPC client the
//! stream verbs use to reach a sandbox's sessions through the broker's
//! `SessionService`.
//!
//! This crate is listed in `.fernignore`, so regeneration never touches it. It
//! is a path dependency of the root, outside any workspace like
//! `hedra-cli-sdk`, through the generator config's `extraDependencies`, and its
//! own tests run through its manifest:
//! `cargo test --manifest-path hedra-cli-sandbox/Cargo.toml --features fake-broker`.

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
#[cfg(feature = "fake-broker")]
pub mod fake_broker;
pub mod session;
pub mod stream;

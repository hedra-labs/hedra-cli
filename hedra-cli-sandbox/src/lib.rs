//! Hand-written support for hedra-cli's sandbox commands.
//!
//! `fern generate` never rewrites or deletes this crate, since `.fernignore`
//! lists it. The root `Cargo.toml` has no `[workspace]` table, so `cargo test`
//! at the repository root does not run this crate's tests; run them through
//! its manifest:
//! `cargo test --manifest-path hedra-cli-sandbox/Cargo.toml --features fake-broker`.

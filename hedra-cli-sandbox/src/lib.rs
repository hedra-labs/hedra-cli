//! Hand-written support for hedra-cli's sandbox commands.
//!
//! This crate is listed in `.fernignore`, so regeneration never touches it. It
//! becomes a path dependency of the root, outside any workspace like
//! `hedra-cli-sdk`, once the generator config's `extraDependencies` names it.
//! Until then nothing builds it but its own tests, run through its manifest:
//! `cargo test --manifest-path hedra-cli-sandbox/Cargo.toml --features fake-broker`.

pub mod stream;

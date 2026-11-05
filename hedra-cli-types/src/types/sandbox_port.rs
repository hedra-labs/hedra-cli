pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The port a sandbox serves at its `url`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxPort {
    /// The port the image serves on: 1 to 65535, other than 18000, which the sandbox reserves.
    #[serde(default)]
    pub port: i64,
    /// An absolute HTTP path on `port` that the readiness probe requests, at most 256 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<String>,
}

impl SandboxPort {
    pub fn builder() -> SandboxPortBuilder {
        <SandboxPortBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxPortBuilder {
    port: Option<i64>,
    health: Option<String>,
}

impl SandboxPortBuilder {
    pub fn port(mut self, value: i64) -> Self {
        self.port = Some(value);
        self
    }

    pub fn health(mut self, value: impl Into<String>) -> Self {
        self.health = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SandboxPort`].
    /// This method will fail if any of the following fields are not set:
    /// - [`port`](SandboxPortBuilder::port)
    pub fn build(self) -> Result<SandboxPort, BuildError> {
        Ok(SandboxPort {
            port: self.port.ok_or_else(|| BuildError::missing_field("port"))?,
            health: self.health,
        })
    }
}

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
    /// Who can call the port. `none`, the default: any client that reaches `url` or `internet_url`, and any authentication is your server's own. `token`: only a client that also sends an endpoint token from `POST /sandboxes/{sandbox}/endpoint-token` with each request. Each create sets it: a create that leaves it out serves the port to any caller, also when an earlier sandbox of the same name used `token`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<SandboxPortAuth>,
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
    auth: Option<SandboxPortAuth>,
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

    pub fn auth(mut self, value: SandboxPortAuth) -> Self {
        self.auth = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxPort`].
    /// This method will fail if any of the following fields are not set:
    /// - [`port`](SandboxPortBuilder::port)
    pub fn build(self) -> Result<SandboxPort, BuildError> {
        Ok(SandboxPort {
            port: self.port.ok_or_else(|| BuildError::missing_field("port"))?,
            health: self.health,
            auth: self.auth,
        })
    }
}

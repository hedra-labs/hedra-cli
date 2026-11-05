pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A session JWT for one instance's streams on the broker: `Logs` always,
/// `PortForward` when the calling key holds `sandboxes:write`, `Exec` and
/// `Invoke` when it holds `sandboxes:exec`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxSession {
    /// The session JWT.
    #[serde(default)]
    pub token: String,
    /// The broker to open streams on with `token`, as `host:port`.
    #[serde(default)]
    pub broker_url: String,
    /// When the broker stops accepting `token` for a new stream. A stream already open keeps running.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub expires_at: DateTime<FixedOffset>,
    /// The replica the session is bound to.
    #[serde(default)]
    pub replica: i64,
}

impl SandboxSession {
    pub fn builder() -> SandboxSessionBuilder {
        <SandboxSessionBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxSessionBuilder {
    token: Option<String>,
    broker_url: Option<String>,
    expires_at: Option<DateTime<FixedOffset>>,
    replica: Option<i64>,
}

impl SandboxSessionBuilder {
    pub fn token(mut self, value: impl Into<String>) -> Self {
        self.token = Some(value.into());
        self
    }

    pub fn broker_url(mut self, value: impl Into<String>) -> Self {
        self.broker_url = Some(value.into());
        self
    }

    pub fn expires_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.expires_at = Some(value);
        self
    }

    pub fn replica(mut self, value: i64) -> Self {
        self.replica = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxSession`].
    /// This method will fail if any of the following fields are not set:
    /// - [`token`](SandboxSessionBuilder::token)
    /// - [`broker_url`](SandboxSessionBuilder::broker_url)
    /// - [`expires_at`](SandboxSessionBuilder::expires_at)
    /// - [`replica`](SandboxSessionBuilder::replica)
    pub fn build(self) -> Result<SandboxSession, BuildError> {
        Ok(SandboxSession {
            token: self.token.ok_or_else(|| BuildError::missing_field("token"))?,
            broker_url: self.broker_url.ok_or_else(|| BuildError::missing_field("broker_url"))?,
            expires_at: self.expires_at.ok_or_else(|| BuildError::missing_field("expires_at"))?,
            replica: self.replica.ok_or_else(|| BuildError::missing_field("replica"))?,
        })
    }
}

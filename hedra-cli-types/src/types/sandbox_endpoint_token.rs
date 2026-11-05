pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A token for a sandbox's `url`, sent as `Authorization: Bearer`, and the
/// link that opens `url` in a browser.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxEndpointToken {
    /// The endpoint token.
    #[serde(default)]
    pub token: String,
    /// The sandbox's `url`, the address to send `token` to.
    #[serde(default)]
    pub url: String,
    /// The link to open in a browser: `url` with `token` as its `hedra_token` query parameter. The sandbox answers the first request with a cookie that holds `token` and a redirect to the same address without the parameter, so that browser stays signed in to this sandbox until `expires_at`, or until a signing-key rotation revokes `token`. It is a credential, as `token` is: anyone who opens it reaches the sandbox until then.
    #[serde(default)]
    pub browser_url: String,
    /// When `token` stops working: the end of the sandbox's `ttl_seconds` or 30 days after the mint, whichever is earlier, later only by the time the mint took. Mint a new one before then.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub expires_at: DateTime<FixedOffset>,
}

impl SandboxEndpointToken {
    pub fn builder() -> SandboxEndpointTokenBuilder {
        <SandboxEndpointTokenBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxEndpointTokenBuilder {
    token: Option<String>,
    url: Option<String>,
    browser_url: Option<String>,
    expires_at: Option<DateTime<FixedOffset>>,
}

impl SandboxEndpointTokenBuilder {
    pub fn token(mut self, value: impl Into<String>) -> Self {
        self.token = Some(value.into());
        self
    }

    pub fn url(mut self, value: impl Into<String>) -> Self {
        self.url = Some(value.into());
        self
    }

    pub fn browser_url(mut self, value: impl Into<String>) -> Self {
        self.browser_url = Some(value.into());
        self
    }

    pub fn expires_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.expires_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxEndpointToken`].
    /// This method will fail if any of the following fields are not set:
    /// - [`token`](SandboxEndpointTokenBuilder::token)
    /// - [`url`](SandboxEndpointTokenBuilder::url)
    /// - [`browser_url`](SandboxEndpointTokenBuilder::browser_url)
    /// - [`expires_at`](SandboxEndpointTokenBuilder::expires_at)
    pub fn build(self) -> Result<SandboxEndpointToken, BuildError> {
        Ok(SandboxEndpointToken {
            token: self.token.ok_or_else(|| BuildError::missing_field("token"))?,
            url: self.url.ok_or_else(|| BuildError::missing_field("url"))?,
            browser_url: self.browser_url.ok_or_else(|| BuildError::missing_field("browser_url"))?,
            expires_at: self.expires_at.ok_or_else(|| BuildError::missing_field("expires_at"))?,
        })
    }
}

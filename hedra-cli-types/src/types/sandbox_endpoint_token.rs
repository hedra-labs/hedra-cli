pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A token for a sandbox's `url`, sent as `Authorization: Bearer`, and the
/// link that opens `url` in a browser; with `internet_url`, the same pair for
/// the public address.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxEndpointToken {
    /// The endpoint token.
    #[serde(default)]
    pub token: String,
    /// The sandbox's `url`, the address to send `token` to.
    #[serde(default)]
    pub url: String,
    /// The link that opens a port with `auth` `token` in a browser: `url` with `token` as its `hedra_token` query parameter. The sandbox answers the first request with a cookie that holds `token` and a redirect to the same address without the parameter. The endpoint token and link reach this sandbox and later sandboxes of the same sandbox name in the same workspace by the same owner until the original `expires_at`, or until a signing-key rotation revokes `token`. Stopping, recreating or changing the image does not revoke the endpoint token; another sandbox name isolates the replacement without revoking the earlier endpoint token. The link is a credential, as `token` is: anyone who opens it reaches the sandboxes in that scope until then. A port with `auth` `none` needs no link: a browser opens `url` itself.
    #[serde(default)]
    pub browser_url: String,
    /// The sandbox's `internet_url`, which takes `token` as `url` does; null while the sandbox has none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internet_url: Option<String>,
    /// `browser_url` for `internet_url`: `internet_url` with `token` as its `hedra_token` query parameter, a credential as `browser_url` is; null while the sandbox has no `internet_url`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub internet_browser_url: Option<String>,
    /// When `token` stops working: 30 days after the mint, later only by the time the mint took, whatever the sandbox's end time. Mint a new one before then.
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
    internet_url: Option<String>,
    internet_browser_url: Option<String>,
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

    pub fn internet_url(mut self, value: impl Into<String>) -> Self {
        self.internet_url = Some(value.into());
        self
    }

    pub fn internet_browser_url(mut self, value: impl Into<String>) -> Self {
        self.internet_browser_url = Some(value.into());
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
            internet_url: self.internet_url,
            internet_browser_url: self.internet_browser_url,
            expires_at: self.expires_at.ok_or_else(|| BuildError::missing_field("expires_at"))?,
        })
    }
}

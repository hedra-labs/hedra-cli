pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The ed25519 public key callers verify outbound webhook signatures with.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct WebhookPublicKey {
    /// Signature algorithm; always `ed25519`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
    /// Base64-encoded ed25519 public key. Verify each delivery's signature header with it before trusting the payload.
    #[serde(default)]
    pub public_key: String,
}

impl WebhookPublicKey {
    pub fn builder() -> WebhookPublicKeyBuilder {
        <WebhookPublicKeyBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct WebhookPublicKeyBuilder {
    algorithm: Option<String>,
    public_key: Option<String>,
}

impl WebhookPublicKeyBuilder {
    pub fn algorithm(mut self, value: impl Into<String>) -> Self {
        self.algorithm = Some(value.into());
        self
    }

    pub fn public_key(mut self, value: impl Into<String>) -> Self {
        self.public_key = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`WebhookPublicKey`].
    /// This method will fail if any of the following fields are not set:
    /// - [`public_key`](WebhookPublicKeyBuilder::public_key)
    pub fn build(self) -> Result<WebhookPublicKey, BuildError> {
        Ok(WebhookPublicKey {
            algorithm: self.algorithm,
            public_key: self.public_key.ok_or_else(|| BuildError::missing_field("public_key"))?,
        })
    }
}

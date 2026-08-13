pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct TokenCreateRequest {
    /// Seconds until the token expires; omitted means the service default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<i64>,
    /// Scopes granted to the token. Omitted means every scope of the minting key; an explicit subset narrows the grant, and requesting beyond the key's scopes is a 403.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<ApiKeyScope>>,
}

impl TokenCreateRequest {
    pub fn builder() -> TokenCreateRequestBuilder {
        <TokenCreateRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct TokenCreateRequestBuilder {
    ttl_seconds: Option<i64>,
    scopes: Option<Vec<ApiKeyScope>>,
}

impl TokenCreateRequestBuilder {
    pub fn ttl_seconds(mut self, value: i64) -> Self {
        self.ttl_seconds = Some(value);
        self
    }

    pub fn scopes(mut self, value: Vec<ApiKeyScope>) -> Self {
        self.scopes = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`TokenCreateRequest`].
    pub fn build(self) -> Result<TokenCreateRequest, BuildError> {
        Ok(TokenCreateRequest {
            ttl_seconds: self.ttl_seconds,
            scopes: self.scopes,
        })
    }
}


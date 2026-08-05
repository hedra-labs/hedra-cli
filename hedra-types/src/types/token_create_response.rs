pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct TokenCreateResponse {
    /// The ephemeral token — a valid Bearer credential until it expires. Shown exactly once.
    #[serde(default)]
    pub token: String,
    /// ISO-8601 instant the token stops authenticating.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<FixedOffset>>,
}

impl TokenCreateResponse {
    pub fn builder() -> TokenCreateResponseBuilder {
        <TokenCreateResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct TokenCreateResponseBuilder {
    token: Option<String>,
    expires_at: Option<DateTime<FixedOffset>>,
}

impl TokenCreateResponseBuilder {
    pub fn token(mut self, value: impl Into<String>) -> Self {
        self.token = Some(value.into());
        self
    }

    pub fn expires_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.expires_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`TokenCreateResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`token`](TokenCreateResponseBuilder::token)
    pub fn build(self) -> Result<TokenCreateResponse, BuildError> {
        Ok(TokenCreateResponse {
            token: self.token.ok_or_else(|| BuildError::missing_field("token"))?,
            expires_at: self.expires_at,
        })
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KeyRotateResponse {
    /// The key's public identifier — unchanged by rotation.
    #[serde(default)]
    pub key_id: String,
    /// The new `<key_id>:<secret>` pair — a valid Bearer credential, shown exactly once, like at create.
    #[serde(default)]
    pub credential: String,
    pub kind: ApiKeyKind,
    /// Human-readable label for the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Scopes granted to the key; null means full access (a legacy key predating scopes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    /// The workspace the key bills and acts in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// ISO-8601 instant the key stops authenticating; null means it never expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<FixedOffset>>,
    /// ISO-8601 instant the previous secret stops authenticating — the end of the grace window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_secret_expires_at: Option<DateTime<FixedOffset>>,
}

impl KeyRotateResponse {
    pub fn builder() -> KeyRotateResponseBuilder {
        <KeyRotateResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct KeyRotateResponseBuilder {
    key_id: Option<String>,
    credential: Option<String>,
    kind: Option<ApiKeyKind>,
    name: Option<String>,
    scopes: Option<Vec<String>>,
    workspace_id: Option<String>,
    expires_at: Option<DateTime<FixedOffset>>,
    previous_secret_expires_at: Option<DateTime<FixedOffset>>,
}

impl KeyRotateResponseBuilder {
    pub fn key_id(mut self, value: impl Into<String>) -> Self {
        self.key_id = Some(value.into());
        self
    }

    pub fn credential(mut self, value: impl Into<String>) -> Self {
        self.credential = Some(value.into());
        self
    }

    pub fn kind(mut self, value: ApiKeyKind) -> Self {
        self.kind = Some(value);
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn scopes(mut self, value: Vec<String>) -> Self {
        self.scopes = Some(value);
        self
    }

    pub fn workspace_id(mut self, value: impl Into<String>) -> Self {
        self.workspace_id = Some(value.into());
        self
    }

    pub fn expires_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.expires_at = Some(value);
        self
    }

    pub fn previous_secret_expires_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.previous_secret_expires_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`KeyRotateResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`key_id`](KeyRotateResponseBuilder::key_id)
    /// - [`credential`](KeyRotateResponseBuilder::credential)
    /// - [`kind`](KeyRotateResponseBuilder::kind)
    pub fn build(self) -> Result<KeyRotateResponse, BuildError> {
        Ok(KeyRotateResponse {
            key_id: self.key_id.ok_or_else(|| BuildError::missing_field("key_id"))?,
            credential: self.credential.ok_or_else(|| BuildError::missing_field("credential"))?,
            kind: self.kind.ok_or_else(|| BuildError::missing_field("kind"))?,
            name: self.name,
            scopes: self.scopes,
            workspace_id: self.workspace_id,
            expires_at: self.expires_at,
            previous_secret_expires_at: self.previous_secret_expires_at,
        })
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KeyCreateResponse {
    #[serde(default)]
    pub key_id: String,
    #[serde(default)]
    pub credential: String,
    pub kind: ApiKeyKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<FixedOffset>>,
}

impl KeyCreateResponse {
    pub fn builder() -> KeyCreateResponseBuilder {
        <KeyCreateResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct KeyCreateResponseBuilder {
    key_id: Option<String>,
    credential: Option<String>,
    kind: Option<ApiKeyKind>,
    name: Option<String>,
    scopes: Option<Vec<String>>,
    workspace_id: Option<String>,
    expires_at: Option<DateTime<FixedOffset>>,
    created_at: Option<DateTime<FixedOffset>>,
}

impl KeyCreateResponseBuilder {
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

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`KeyCreateResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`key_id`](KeyCreateResponseBuilder::key_id)
    /// - [`credential`](KeyCreateResponseBuilder::credential)
    /// - [`kind`](KeyCreateResponseBuilder::kind)
    pub fn build(self) -> Result<KeyCreateResponse, BuildError> {
        Ok(KeyCreateResponse {
            key_id: self.key_id.ok_or_else(|| BuildError::missing_field("key_id"))?,
            credential: self.credential.ok_or_else(|| BuildError::missing_field("credential"))?,
            kind: self.kind.ok_or_else(|| BuildError::missing_field("kind"))?,
            name: self.name,
            scopes: self.scopes,
            workspace_id: self.workspace_id,
            expires_at: self.expires_at,
            created_at: self.created_at,
        })
    }
}

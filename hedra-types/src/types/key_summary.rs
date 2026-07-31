pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KeySummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
    pub kind: ApiKeyKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    pub status: KeyStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revoked_at: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<FixedOffset>>,
}

impl KeySummary {
    pub fn builder() -> KeySummaryBuilder {
        <KeySummaryBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct KeySummaryBuilder {
    key_id: Option<String>,
    kind: Option<ApiKeyKind>,
    name: Option<String>,
    scopes: Option<Vec<String>>,
    workspace_id: Option<String>,
    status: Option<KeyStatus>,
    created_at: Option<DateTime<FixedOffset>>,
    expires_at: Option<DateTime<FixedOffset>>,
    revoked_at: Option<DateTime<FixedOffset>>,
    last_used_at: Option<DateTime<FixedOffset>>,
}

impl KeySummaryBuilder {
    pub fn key_id(mut self, value: impl Into<String>) -> Self {
        self.key_id = Some(value.into());
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

    pub fn status(mut self, value: KeyStatus) -> Self {
        self.status = Some(value);
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn expires_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.expires_at = Some(value);
        self
    }

    pub fn revoked_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.revoked_at = Some(value);
        self
    }

    pub fn last_used_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.last_used_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`KeySummary`].
    /// This method will fail if any of the following fields are not set:
    /// - [`kind`](KeySummaryBuilder::kind)
    /// - [`status`](KeySummaryBuilder::status)
    pub fn build(self) -> Result<KeySummary, BuildError> {
        Ok(KeySummary {
            key_id: self.key_id,
            kind: self.kind.ok_or_else(|| BuildError::missing_field("kind"))?,
            name: self.name,
            scopes: self.scopes,
            workspace_id: self.workspace_id,
            status: self.status.ok_or_else(|| BuildError::missing_field("status"))?,
            created_at: self.created_at,
            expires_at: self.expires_at,
            revoked_at: self.revoked_at,
            last_used_at: self.last_used_at,
        })
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct KeyCreateRequest {
    /// Human-readable label for the key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Scopes granted to the key; omitted means full access.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<ApiKeyScope>>,
    /// `personal` (default) dies with the member; `service` is workspace-shared, OWNER/ADMIN-managed, and survives member removal.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<ApiKeyKind>,
    /// Target workspace; omitted means the authenticating key's workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// ISO-8601 instant the key stops authenticating; omitted means it never expires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<FixedOffset>>,
}

impl KeyCreateRequest {
    pub fn builder() -> KeyCreateRequestBuilder {
        <KeyCreateRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct KeyCreateRequestBuilder {
    name: Option<String>,
    scopes: Option<Vec<ApiKeyScope>>,
    kind: Option<ApiKeyKind>,
    workspace_id: Option<String>,
    expires_at: Option<DateTime<FixedOffset>>,
}

impl KeyCreateRequestBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn scopes(mut self, value: Vec<ApiKeyScope>) -> Self {
        self.scopes = Some(value);
        self
    }

    pub fn kind(mut self, value: ApiKeyKind) -> Self {
        self.kind = Some(value);
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

    /// Consumes the builder and constructs a [`KeyCreateRequest`].
    pub fn build(self) -> Result<KeyCreateRequest, BuildError> {
        Ok(KeyCreateRequest {
            name: self.name,
            scopes: self.scopes,
            kind: self.kind,
            workspace_id: self.workspace_id,
            expires_at: self.expires_at,
        })
    }
}


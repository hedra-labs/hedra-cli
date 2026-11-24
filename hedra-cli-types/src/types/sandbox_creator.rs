pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Who created a sandbox or a job, a workspace member or the workspace's
/// service key.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SandboxCreator {
    /// The creator's email; null for a workspace service key and for a creator whose account was deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// The creator's first and last name; null when they have set neither, for a workspace service key and for a creator whose account was deleted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// `service` when the workspace's service key created it; `personal` when a member did, with their own key or session, and for a creator whose account was deleted.
    pub kind: ApiKeyKind,
}

impl SandboxCreator {
    pub fn builder() -> SandboxCreatorBuilder {
        <SandboxCreatorBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxCreatorBuilder {
    email: Option<String>,
    name: Option<String>,
    kind: Option<ApiKeyKind>,
}

impl SandboxCreatorBuilder {
    pub fn email(mut self, value: impl Into<String>) -> Self {
        self.email = Some(value.into());
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn kind(mut self, value: ApiKeyKind) -> Self {
        self.kind = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxCreator`].
    /// This method will fail if any of the following fields are not set:
    /// - [`kind`](SandboxCreatorBuilder::kind)
    pub fn build(self) -> Result<SandboxCreator, BuildError> {
        Ok(SandboxCreator {
            email: self.email,
            name: self.name,
            kind: self.kind.ok_or_else(|| BuildError::missing_field("kind"))?,
        })
    }
}

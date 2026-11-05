pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A workspace secret: its name only, since the value never comes back.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxSecret {
    /// The secret's name.
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
}

impl SandboxSecret {
    pub fn builder() -> SandboxSecretBuilder {
        <SandboxSecretBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxSecretBuilder {
    name: Option<String>,
    created_at: Option<DateTime<FixedOffset>>,
}

impl SandboxSecretBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxSecret`].
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](SandboxSecretBuilder::name)
    /// - [`created_at`](SandboxSecretBuilder::created_at)
    pub fn build(self) -> Result<SandboxSecret, BuildError> {
        Ok(SandboxSecret {
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
        })
    }
}

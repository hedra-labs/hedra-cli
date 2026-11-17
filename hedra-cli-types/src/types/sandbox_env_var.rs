pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A workspace environment variable, without its value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SandboxEnvVar {
    /// The variable's name.
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub created_at: DateTime<FixedOffset>,
    /// When its value was last set; null for a secret last set before this was recorded.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset::option")]
    pub updated_at: Option<DateTime<FixedOffset>>,
    pub r#type: SandboxEnvVarType,
}

impl SandboxEnvVar {
    pub fn builder() -> SandboxEnvVarBuilder {
        <SandboxEnvVarBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxEnvVarBuilder {
    name: Option<String>,
    created_at: Option<DateTime<FixedOffset>>,
    updated_at: Option<DateTime<FixedOffset>>,
    r#type: Option<SandboxEnvVarType>,
}

impl SandboxEnvVarBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn created_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_at = Some(value);
        self
    }

    pub fn updated_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.updated_at = Some(value);
        self
    }

    pub fn r#type(mut self, value: SandboxEnvVarType) -> Self {
        self.r#type = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxEnvVar`].
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](SandboxEnvVarBuilder::name)
    /// - [`created_at`](SandboxEnvVarBuilder::created_at)
    /// - [`r#type`](SandboxEnvVarBuilder::r#type)
    pub fn build(self) -> Result<SandboxEnvVar, BuildError> {
        Ok(SandboxEnvVar {
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            created_at: self.created_at.ok_or_else(|| BuildError::missing_field("created_at"))?,
            updated_at: self.updated_at,
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
        })
    }
}

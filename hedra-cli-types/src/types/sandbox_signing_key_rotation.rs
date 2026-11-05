pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A rotation of the key that signs your workspace's session JWTs and
/// endpoint tokens.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxSigningKeyRotation {
    /// When the new key took over. Every session JWT and endpoint token signed before it stops working within 30 seconds; a stream or request already open keeps running.
    #[serde(default)]
    #[serde(with = "crate::core::flexible_datetime::offset")]
    pub rotated_at: DateTime<FixedOffset>,
}

impl SandboxSigningKeyRotation {
    pub fn builder() -> SandboxSigningKeyRotationBuilder {
        <SandboxSigningKeyRotationBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxSigningKeyRotationBuilder {
    rotated_at: Option<DateTime<FixedOffset>>,
}

impl SandboxSigningKeyRotationBuilder {
    pub fn rotated_at(mut self, value: DateTime<FixedOffset>) -> Self {
        self.rotated_at = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxSigningKeyRotation`].
    /// This method will fail if any of the following fields are not set:
    /// - [`rotated_at`](SandboxSigningKeyRotationBuilder::rotated_at)
    pub fn build(self) -> Result<SandboxSigningKeyRotation, BuildError> {
        Ok(SandboxSigningKeyRotation {
            rotated_at: self.rotated_at.ok_or_else(|| BuildError::missing_field("rotated_at"))?,
        })
    }
}

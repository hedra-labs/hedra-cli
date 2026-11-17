pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// A saved workspace environment variable, Config or Secret, by name:
/// each instance reads its value when it starts.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct SandboxEnvRef {
    /// The name of a variable `GET /sandboxes/environment-variables` lists.
    pub r#ref: String,
}

impl SandboxEnvRef {
    pub fn builder() -> SandboxEnvRefBuilder {
        <SandboxEnvRefBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxEnvRefBuilder {
    r#ref: Option<String>,
}

impl SandboxEnvRefBuilder {
    pub fn r#ref(mut self, value: impl Into<String>) -> Self {
        self.r#ref = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SandboxEnvRef`].
    /// This method will fail if any of the following fields are not set:
    /// - [`r#ref`](SandboxEnvRefBuilder::r#ref)
    pub fn build(self) -> Result<SandboxEnvRef, BuildError> {
        Ok(SandboxEnvRef {
            r#ref: self.r#ref.ok_or_else(|| BuildError::missing_field("r#ref"))?,
        })
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxEnvVarListResponse {
    /// Every variable your workspace holds of the type asked for, by name, without values.
    #[serde(default)]
    pub data: Vec<SandboxEnvVar>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl SandboxEnvVarListResponse {
    pub fn builder() -> SandboxEnvVarListResponseBuilder {
        <SandboxEnvVarListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxEnvVarListResponseBuilder {
    data: Option<Vec<SandboxEnvVar>>,
    next_cursor: Option<String>,
}

impl SandboxEnvVarListResponseBuilder {
    pub fn data(mut self, value: Vec<SandboxEnvVar>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SandboxEnvVarListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](SandboxEnvVarListResponseBuilder::data)
    pub fn build(self) -> Result<SandboxEnvVarListResponse, BuildError> {
        Ok(SandboxEnvVarListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

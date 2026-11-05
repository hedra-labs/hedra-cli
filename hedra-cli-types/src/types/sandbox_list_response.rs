pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SandboxListResponse {
    /// Your sandboxes, newest first.
    #[serde(default)]
    pub data: Vec<Sandbox>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl SandboxListResponse {
    pub fn builder() -> SandboxListResponseBuilder {
        <SandboxListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxListResponseBuilder {
    data: Option<Vec<Sandbox>>,
    next_cursor: Option<String>,
}

impl SandboxListResponseBuilder {
    pub fn data(mut self, value: Vec<Sandbox>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SandboxListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](SandboxListResponseBuilder::data)
    pub fn build(self) -> Result<SandboxListResponse, BuildError> {
        Ok(SandboxListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

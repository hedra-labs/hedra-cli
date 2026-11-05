pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SandboxGpuListResponse {
    /// Every GPU model you can run.
    #[serde(default)]
    pub data: Vec<SandboxGpu>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl SandboxGpuListResponse {
    pub fn builder() -> SandboxGpuListResponseBuilder {
        <SandboxGpuListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxGpuListResponseBuilder {
    data: Option<Vec<SandboxGpu>>,
    next_cursor: Option<String>,
}

impl SandboxGpuListResponseBuilder {
    pub fn data(mut self, value: Vec<SandboxGpu>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SandboxGpuListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](SandboxGpuListResponseBuilder::data)
    pub fn build(self) -> Result<SandboxGpuListResponse, BuildError> {
        Ok(SandboxGpuListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

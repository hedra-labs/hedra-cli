pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct LogDrainListResponse {
    #[serde(default)]
    pub data: Vec<LogDrainConfig>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl LogDrainListResponse {
    pub fn builder() -> LogDrainListResponseBuilder {
        <LogDrainListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LogDrainListResponseBuilder {
    data: Option<Vec<LogDrainConfig>>,
    next_cursor: Option<String>,
}

impl LogDrainListResponseBuilder {
    pub fn data(mut self, value: Vec<LogDrainConfig>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`LogDrainListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](LogDrainListResponseBuilder::data)
    pub fn build(self) -> Result<LogDrainListResponse, BuildError> {
        Ok(LogDrainListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

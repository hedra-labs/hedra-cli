pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct LlmUsageRecordListResponse {
    /// This page of items.
    #[serde(default)]
    pub data: Vec<LlmUsageRecord>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl LlmUsageRecordListResponse {
    pub fn builder() -> LlmUsageRecordListResponseBuilder {
        <LlmUsageRecordListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LlmUsageRecordListResponseBuilder {
    data: Option<Vec<LlmUsageRecord>>,
    next_cursor: Option<String>,
}

impl LlmUsageRecordListResponseBuilder {
    pub fn data(mut self, value: Vec<LlmUsageRecord>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`LlmUsageRecordListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](LlmUsageRecordListResponseBuilder::data)
    pub fn build(self) -> Result<LlmUsageRecordListResponse, BuildError> {
        Ok(LlmUsageRecordListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

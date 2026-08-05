pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct JobLogListResponse {
    /// This page of items.
    #[serde(default)]
    pub data: Vec<JobLogItem>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl JobLogListResponse {
    pub fn builder() -> JobLogListResponseBuilder {
        <JobLogListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct JobLogListResponseBuilder {
    data: Option<Vec<JobLogItem>>,
    next_cursor: Option<String>,
}

impl JobLogListResponseBuilder {
    pub fn data(mut self, value: Vec<JobLogItem>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`JobLogListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](JobLogListResponseBuilder::data)
    pub fn build(self) -> Result<JobLogListResponse, BuildError> {
        Ok(JobLogListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

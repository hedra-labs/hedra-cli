pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct JobListResponse {
    #[serde(default)]
    pub data: Vec<JobSummary>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl JobListResponse {
    pub fn builder() -> JobListResponseBuilder {
        <JobListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct JobListResponseBuilder {
    data: Option<Vec<JobSummary>>,
    next_cursor: Option<String>,
}

impl JobListResponseBuilder {
    pub fn data(mut self, value: Vec<JobSummary>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`JobListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](JobListResponseBuilder::data)
    pub fn build(self) -> Result<JobListResponse, BuildError> {
        Ok(JobListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

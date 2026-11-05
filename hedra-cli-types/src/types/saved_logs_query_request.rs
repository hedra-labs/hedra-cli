pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for saved-logs
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SavedLogsQueryRequest {
    /// The instance's replica number, from 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replica: Option<i64>,
    /// Only output written at or after this instant: an RFC 3339 timestamp with an offset and up to nanoseconds, such as `2026-11-02T18:32:30.123456789Z`. It is rounded up to a whole microsecond.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}

impl SavedLogsQueryRequest {
    pub fn builder() -> SavedLogsQueryRequestBuilder {
        <SavedLogsQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SavedLogsQueryRequestBuilder {
    replica: Option<i64>,
    since: Option<String>,
}

impl SavedLogsQueryRequestBuilder {
    pub fn replica(mut self, value: i64) -> Self {
        self.replica = Some(value);
        self
    }

    pub fn since(mut self, value: impl Into<String>) -> Self {
        self.since = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SavedLogsQueryRequest`].
    pub fn build(self) -> Result<SavedLogsQueryRequest, BuildError> {
        Ok(SavedLogsQueryRequest {
            replica: self.replica,
            since: self.since,
        })
    }
}


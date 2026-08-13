pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for getStatus
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct GetStatusQueryRequest {
    /// Tail this job's lifecycle events incrementally: returns only events newer than this cursor, plus `logs_next_cursor` to send on the next poll. Pass `start` to begin from the job's first event. Omit it and the response carries no events at all — the default polling shape is unchanged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs_after: Option<String>,
}

impl GetStatusQueryRequest {
    pub fn builder() -> GetStatusQueryRequestBuilder {
        <GetStatusQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GetStatusQueryRequestBuilder {
    logs_after: Option<String>,
}

impl GetStatusQueryRequestBuilder {
    pub fn logs_after(mut self, value: impl Into<String>) -> Self {
        self.logs_after = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`GetStatusQueryRequest`].
    pub fn build(self) -> Result<GetStatusQueryRequest, BuildError> {
        Ok(GetStatusQueryRequest {
            logs_after: self.logs_after,
        })
    }
}


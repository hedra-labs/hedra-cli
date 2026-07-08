pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct RequestListResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<RequestSummary>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl RequestListResponse {
    pub fn builder() -> RequestListResponseBuilder {
        <RequestListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct RequestListResponseBuilder {
    data: Option<Vec<RequestSummary>>,
    next_cursor: Option<String>,
}

impl RequestListResponseBuilder {
    pub fn data(mut self, value: Vec<RequestSummary>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`RequestListResponse`].
    pub fn build(self) -> Result<RequestListResponse, BuildError> {
        Ok(RequestListResponse {
            data: self.data,
            next_cursor: self.next_cursor,
        })
    }
}

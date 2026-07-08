pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for list
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct RequestsListQueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl RequestsListQueryRequest {
    pub fn builder() -> RequestsListQueryRequestBuilder {
        <RequestsListQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct RequestsListQueryRequestBuilder {
    limit: Option<i64>,
    cursor: Option<String>,
}

impl RequestsListQueryRequestBuilder {
    pub fn limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn cursor(mut self, value: impl Into<String>) -> Self {
        self.cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`RequestsListQueryRequest`].
    pub fn build(self) -> Result<RequestsListQueryRequest, BuildError> {
        Ok(RequestsListQueryRequest {
            limit: self.limit,
            cursor: self.cursor,
        })
    }
}


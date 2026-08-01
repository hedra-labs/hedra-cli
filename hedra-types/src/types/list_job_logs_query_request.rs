pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for listJobLogs
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ListJobLogsQueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl ListJobLogsQueryRequest {
    pub fn builder() -> ListJobLogsQueryRequestBuilder {
        <ListJobLogsQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ListJobLogsQueryRequestBuilder {
    limit: Option<i64>,
    cursor: Option<String>,
}

impl ListJobLogsQueryRequestBuilder {
    pub fn limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn cursor(mut self, value: impl Into<String>) -> Self {
        self.cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`ListJobLogsQueryRequest`].
    pub fn build(self) -> Result<ListJobLogsQueryRequest, BuildError> {
        Ok(ListJobLogsQueryRequest {
            limit: self.limit,
            cursor: self.cursor,
        })
    }
}


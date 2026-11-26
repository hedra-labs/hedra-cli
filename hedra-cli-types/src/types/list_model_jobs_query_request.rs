pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for listModelJobs
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ListModelJobsQueryRequest {
    /// Maximum items per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Opaque cursor from the previous page's `next_cursor`; omit for the first page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// With `true`, the default, every job for this model in your workspace, whoever submitted it; with `false`, only the jobs you submitted. Any member of the workspace may pass either.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<bool>,
}

impl ListModelJobsQueryRequest {
    pub fn builder() -> ListModelJobsQueryRequestBuilder {
        <ListModelJobsQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ListModelJobsQueryRequestBuilder {
    limit: Option<i64>,
    cursor: Option<String>,
    members: Option<bool>,
}

impl ListModelJobsQueryRequestBuilder {
    pub fn limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn cursor(mut self, value: impl Into<String>) -> Self {
        self.cursor = Some(value.into());
        self
    }

    pub fn members(mut self, value: bool) -> Self {
        self.members = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ListModelJobsQueryRequest`].
    pub fn build(self) -> Result<ListModelJobsQueryRequest, BuildError> {
        Ok(ListModelJobsQueryRequest {
            limit: self.limit,
            cursor: self.cursor,
            members: self.members,
        })
    }
}


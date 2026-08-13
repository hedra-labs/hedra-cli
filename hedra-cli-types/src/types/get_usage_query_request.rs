pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for getUsage
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct GetUsageQueryRequest {
    /// Window start (inclusive, ISO-8601); defaults to 7 days before `end`. Bounds job-creation time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<FixedOffset>>,
    /// Window end (exclusive, ISO-8601); defaults to now. The window is capped at 90 days.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<FixedOffset>>,
    /// One summary row (`total`), one per UTC day (`day`), or one per model (`model`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_by: Option<UsageGroupBy>,
}

impl GetUsageQueryRequest {
    pub fn builder() -> GetUsageQueryRequestBuilder {
        <GetUsageQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GetUsageQueryRequestBuilder {
    start: Option<DateTime<FixedOffset>>,
    end: Option<DateTime<FixedOffset>>,
    group_by: Option<UsageGroupBy>,
}

impl GetUsageQueryRequestBuilder {
    pub fn start(mut self, value: DateTime<FixedOffset>) -> Self {
        self.start = Some(value);
        self
    }

    pub fn end(mut self, value: DateTime<FixedOffset>) -> Self {
        self.end = Some(value);
        self
    }

    pub fn group_by(mut self, value: UsageGroupBy) -> Self {
        self.group_by = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`GetUsageQueryRequest`].
    pub fn build(self) -> Result<GetUsageQueryRequest, BuildError> {
        Ok(GetUsageQueryRequest {
            start: self.start,
            end: self.end,
            group_by: self.group_by,
        })
    }
}


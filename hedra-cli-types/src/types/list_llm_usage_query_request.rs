pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for listLlmUsage
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ListLlmUsageQueryRequest {
    /// Maximum items per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Opaque cursor from the previous page's `next_cursor`; omit for the first page. Each cursor records the filters it was issued under, so a walk must keep them fixed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Only requests created at or after this instant (ISO-8601). Unlike `GET /v3/usage` there is no default window and no window cap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<DateTime<FixedOffset>>,
    /// Only requests created before this instant (ISO-8601, exclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<DateTime<FixedOffset>>,
    /// Only requests admitted for this model id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Only requests made with this API key (its `key_id`, as listed by GET /v3/keys). An unknown key id yields an empty page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_id: Option<String>,
}

impl ListLlmUsageQueryRequest {
    pub fn builder() -> ListLlmUsageQueryRequestBuilder {
        <ListLlmUsageQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ListLlmUsageQueryRequestBuilder {
    limit: Option<i64>,
    cursor: Option<String>,
    start: Option<DateTime<FixedOffset>>,
    end: Option<DateTime<FixedOffset>>,
    model: Option<String>,
    key_id: Option<String>,
}

impl ListLlmUsageQueryRequestBuilder {
    pub fn limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn cursor(mut self, value: impl Into<String>) -> Self {
        self.cursor = Some(value.into());
        self
    }

    pub fn start(mut self, value: DateTime<FixedOffset>) -> Self {
        self.start = Some(value);
        self
    }

    pub fn end(mut self, value: DateTime<FixedOffset>) -> Self {
        self.end = Some(value);
        self
    }

    pub fn model(mut self, value: impl Into<String>) -> Self {
        self.model = Some(value.into());
        self
    }

    pub fn key_id(mut self, value: impl Into<String>) -> Self {
        self.key_id = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`ListLlmUsageQueryRequest`].
    pub fn build(self) -> Result<ListLlmUsageQueryRequest, BuildError> {
        Ok(ListLlmUsageQueryRequest {
            limit: self.limit,
            cursor: self.cursor,
            start: self.start,
            end: self.end,
            model: self.model,
            key_id: self.key_id,
        })
    }
}


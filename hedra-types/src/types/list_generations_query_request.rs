pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for list_generations
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ListGenerationsQueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<ListGenerationsRequestType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_before: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_after: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<String>,
    /// Number of items returned in the page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Number of records skipped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

impl ListGenerationsQueryRequest {
    pub fn builder() -> ListGenerationsQueryRequestBuilder {
        <ListGenerationsQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ListGenerationsQueryRequestBuilder {
    r#type: Option<ListGenerationsRequestType>,
    created_before: Option<DateTime<FixedOffset>>,
    created_after: Option<DateTime<FixedOffset>>,
    prompt_query: Option<String>,
    agent_thread_id: Option<String>,
    ids: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

impl ListGenerationsQueryRequestBuilder {
    pub fn r#type(mut self, value: ListGenerationsRequestType) -> Self {
        self.r#type = Some(value);
        self
    }

    pub fn created_before(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_before = Some(value);
        self
    }

    pub fn created_after(mut self, value: DateTime<FixedOffset>) -> Self {
        self.created_after = Some(value);
        self
    }

    pub fn prompt_query(mut self, value: impl Into<String>) -> Self {
        self.prompt_query = Some(value.into());
        self
    }

    pub fn agent_thread_id(mut self, value: impl Into<String>) -> Self {
        self.agent_thread_id = Some(value.into());
        self
    }

    pub fn ids(mut self, value: impl Into<String>) -> Self {
        self.ids = Some(value.into());
        self
    }

    pub fn limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn offset(mut self, value: i64) -> Self {
        self.offset = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`ListGenerationsQueryRequest`].
    pub fn build(self) -> Result<ListGenerationsQueryRequest, BuildError> {
        Ok(ListGenerationsQueryRequest {
            r#type: self.r#type,
            created_before: self.created_before,
            created_after: self.created_after,
            prompt_query: self.prompt_query,
            agent_thread_id: self.agent_thread_id,
            ids: self.ids,
            limit: self.limit,
            offset: self.offset,
        })
    }
}


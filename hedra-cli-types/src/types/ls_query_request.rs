pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for ls
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct LsQueryRequest {
    /// Maximum items per page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Opaque cursor from the previous page's `next_cursor`; omit for the first page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Only sandboxes with exactly this name. A name is unique among your creating, active and stopping sandboxes, so without `all=true` the page holds at most one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// With `true`, sandboxes in every status; with `false`, the default, only creating, active and stopping sandboxes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all: Option<bool>,
}

impl LsQueryRequest {
    pub fn builder() -> LsQueryRequestBuilder {
        <LsQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct LsQueryRequestBuilder {
    limit: Option<i64>,
    cursor: Option<String>,
    name: Option<String>,
    all: Option<bool>,
}

impl LsQueryRequestBuilder {
    pub fn limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn cursor(mut self, value: impl Into<String>) -> Self {
        self.cursor = Some(value.into());
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn all(mut self, value: bool) -> Self {
        self.all = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`LsQueryRequest`].
    pub fn build(self) -> Result<LsQueryRequest, BuildError> {
        Ok(LsQueryRequest {
            limit: self.limit,
            cursor: self.cursor,
            name: self.name,
            all: self.all,
        })
    }
}


pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct PageInfo {
    /// Number of items returned in the page.
    #[serde(default)]
    pub limit: i64,
    /// Number of records skipped.
    #[serde(default)]
    pub offset: i64,
}

impl PageInfo {
    pub fn builder() -> PageInfoBuilder {
        <PageInfoBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct PageInfoBuilder {
    limit: Option<i64>,
    offset: Option<i64>,
}

impl PageInfoBuilder {
    pub fn limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn offset(mut self, value: i64) -> Self {
        self.offset = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`PageInfo`].
    /// This method will fail if any of the following fields are not set:
    /// - [`limit`](PageInfoBuilder::limit)
    /// - [`offset`](PageInfoBuilder::offset)
    pub fn build(self) -> Result<PageInfo, BuildError> {
        Ok(PageInfo {
            limit: self.limit.ok_or_else(|| BuildError::missing_field("limit"))?,
            offset: self.offset.ok_or_else(|| BuildError::missing_field("offset"))?,
        })
    }
}

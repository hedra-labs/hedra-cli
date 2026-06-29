pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct PagingParams {
    /// Number of items returned in the page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Number of records skipped.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

impl PagingParams {
    pub fn builder() -> PagingParamsBuilder {
        <PagingParamsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct PagingParamsBuilder {
    limit: Option<i64>,
    offset: Option<i64>,
}

impl PagingParamsBuilder {
    pub fn limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn offset(mut self, value: i64) -> Self {
        self.offset = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`PagingParams`].
    pub fn build(self) -> Result<PagingParams, BuildError> {
        Ok(PagingParams {
            limit: self.limit,
            offset: self.offset,
        })
    }
}

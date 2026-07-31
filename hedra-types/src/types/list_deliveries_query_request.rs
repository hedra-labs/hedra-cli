pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for listDeliveries
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct ListDeliveriesQueryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

impl ListDeliveriesQueryRequest {
    pub fn builder() -> ListDeliveriesQueryRequestBuilder {
        <ListDeliveriesQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct ListDeliveriesQueryRequestBuilder {
    limit: Option<i64>,
    cursor: Option<String>,
}

impl ListDeliveriesQueryRequestBuilder {
    pub fn limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn cursor(mut self, value: impl Into<String>) -> Self {
        self.cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`ListDeliveriesQueryRequest`].
    pub fn build(self) -> Result<ListDeliveriesQueryRequest, BuildError> {
        Ok(ListDeliveriesQueryRequest {
            limit: self.limit,
            cursor: self.cursor,
        })
    }
}


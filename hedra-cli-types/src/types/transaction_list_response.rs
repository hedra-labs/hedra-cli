pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TransactionListResponse {
    /// This page of items.
    #[serde(default)]
    pub data: Vec<TransactionRecord>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl TransactionListResponse {
    pub fn builder() -> TransactionListResponseBuilder {
        <TransactionListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct TransactionListResponseBuilder {
    data: Option<Vec<TransactionRecord>>,
    next_cursor: Option<String>,
}

impl TransactionListResponseBuilder {
    pub fn data(mut self, value: Vec<TransactionRecord>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`TransactionListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](TransactionListResponseBuilder::data)
    pub fn build(self) -> Result<TransactionListResponse, BuildError> {
        Ok(TransactionListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

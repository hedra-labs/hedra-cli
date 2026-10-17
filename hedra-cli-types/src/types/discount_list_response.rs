pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The negotiated discounts the key's workspace is charged under now.
/// 
/// Every estimate and charge for these models already reflects them.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct DiscountListResponse {
    /// This page of items.
    #[serde(default)]
    pub data: Vec<Discount>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl DiscountListResponse {
    pub fn builder() -> DiscountListResponseBuilder {
        <DiscountListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct DiscountListResponseBuilder {
    data: Option<Vec<Discount>>,
    next_cursor: Option<String>,
}

impl DiscountListResponseBuilder {
    pub fn data(mut self, value: Vec<Discount>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`DiscountListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](DiscountListResponseBuilder::data)
    pub fn build(self) -> Result<DiscountListResponse, BuildError> {
        Ok(DiscountListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct WebhookDeliveryListResponse {
    #[serde(default)]
    pub data: Vec<WebhookDeliverySummary>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl WebhookDeliveryListResponse {
    pub fn builder() -> WebhookDeliveryListResponseBuilder {
        <WebhookDeliveryListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct WebhookDeliveryListResponseBuilder {
    data: Option<Vec<WebhookDeliverySummary>>,
    next_cursor: Option<String>,
}

impl WebhookDeliveryListResponseBuilder {
    pub fn data(mut self, value: Vec<WebhookDeliverySummary>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`WebhookDeliveryListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](WebhookDeliveryListResponseBuilder::data)
    pub fn build(self) -> Result<WebhookDeliveryListResponse, BuildError> {
        Ok(WebhookDeliveryListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

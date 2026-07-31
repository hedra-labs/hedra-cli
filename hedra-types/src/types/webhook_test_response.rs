pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct WebhookTestResponse {
    #[serde(default)]
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_status: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl WebhookTestResponse {
    pub fn builder() -> WebhookTestResponseBuilder {
        <WebhookTestResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct WebhookTestResponseBuilder {
    ok: Option<bool>,
    response_status: Option<i64>,
    error: Option<String>,
}

impl WebhookTestResponseBuilder {
    pub fn ok(mut self, value: bool) -> Self {
        self.ok = Some(value);
        self
    }

    pub fn response_status(mut self, value: i64) -> Self {
        self.response_status = Some(value);
        self
    }

    pub fn error(mut self, value: impl Into<String>) -> Self {
        self.error = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`WebhookTestResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`ok`](WebhookTestResponseBuilder::ok)
    pub fn build(self) -> Result<WebhookTestResponse, BuildError> {
        Ok(WebhookTestResponse {
            ok: self.ok.ok_or_else(|| BuildError::missing_field("ok"))?,
            response_status: self.response_status,
            error: self.error,
        })
    }
}

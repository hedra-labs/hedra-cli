pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
}

impl SubmitRequest {
    pub fn builder() -> SubmitRequestBuilder {
        <SubmitRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitRequestBuilder {
    input: Option<HashMap<String, serde_json::Value>>,
    webhook: Option<String>,
    idempotency_key: Option<String>,
    priority: Option<String>,
}

impl SubmitRequestBuilder {
    pub fn input(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    pub fn idempotency_key(mut self, value: impl Into<String>) -> Self {
        self.idempotency_key = Some(value.into());
        self
    }

    pub fn priority(mut self, value: impl Into<String>) -> Self {
        self.priority = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitRequest`].
    pub fn build(self) -> Result<SubmitRequest, BuildError> {
        Ok(SubmitRequest {
            input: self.input,
            webhook: self.webhook,
            idempotency_key: self.idempotency_key,
            priority: self.priority,
        })
    }
}


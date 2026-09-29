pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct SubmitBody {
    /// Model-specific inputs, validated at submit against the model's published input schema (`GET /v3/models/{model}`).
    #[serde(default)]
    pub input: HashMap<String, serde_json::Value>,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBody {
    pub fn builder() -> SubmitBodyBuilder {
        <SubmitBodyBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyBuilder {
    input: Option<HashMap<String, serde_json::Value>>,
    webhook: Option<String>,
}

impl SubmitBodyBuilder {
    pub fn input(mut self, value: HashMap<String, serde_json::Value>) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBody`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyBuilder::input)
    pub fn build(self) -> Result<SubmitBody, BuildError> {
        Ok(SubmitBody {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


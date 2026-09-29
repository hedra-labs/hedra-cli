pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyCreatifyAurora {
    pub input: InputCreatifyAurora,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyCreatifyAurora {
    pub fn builder() -> SubmitBodyCreatifyAuroraBuilder {
        <SubmitBodyCreatifyAuroraBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyCreatifyAuroraBuilder {
    input: Option<InputCreatifyAurora>,
    webhook: Option<String>,
}

impl SubmitBodyCreatifyAuroraBuilder {
    pub fn input(mut self, value: InputCreatifyAurora) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyCreatifyAurora`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyCreatifyAuroraBuilder::input)
    pub fn build(self) -> Result<SubmitBodyCreatifyAurora, BuildError> {
        Ok(SubmitBodyCreatifyAurora {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


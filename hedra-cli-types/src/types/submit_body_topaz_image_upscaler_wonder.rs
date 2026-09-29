pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyTopazImageUpscalerWonder {
    pub input: InputTopazImageUpscalerWonder,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyTopazImageUpscalerWonder {
    pub fn builder() -> SubmitBodyTopazImageUpscalerWonderBuilder {
        <SubmitBodyTopazImageUpscalerWonderBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyTopazImageUpscalerWonderBuilder {
    input: Option<InputTopazImageUpscalerWonder>,
    webhook: Option<String>,
}

impl SubmitBodyTopazImageUpscalerWonderBuilder {
    pub fn input(mut self, value: InputTopazImageUpscalerWonder) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyTopazImageUpscalerWonder`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyTopazImageUpscalerWonderBuilder::input)
    pub fn build(self) -> Result<SubmitBodyTopazImageUpscalerWonder, BuildError> {
        Ok(SubmitBodyTopazImageUpscalerWonder {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


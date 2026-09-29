pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyTopazImageUpscalerTransparency {
    pub input: InputTopazImageUpscalerTransparency,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyTopazImageUpscalerTransparency {
    pub fn builder() -> SubmitBodyTopazImageUpscalerTransparencyBuilder {
        <SubmitBodyTopazImageUpscalerTransparencyBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyTopazImageUpscalerTransparencyBuilder {
    input: Option<InputTopazImageUpscalerTransparency>,
    webhook: Option<String>,
}

impl SubmitBodyTopazImageUpscalerTransparencyBuilder {
    pub fn input(mut self, value: InputTopazImageUpscalerTransparency) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyTopazImageUpscalerTransparency`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyTopazImageUpscalerTransparencyBuilder::input)
    pub fn build(self) -> Result<SubmitBodyTopazImageUpscalerTransparency, BuildError> {
        Ok(SubmitBodyTopazImageUpscalerTransparency {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


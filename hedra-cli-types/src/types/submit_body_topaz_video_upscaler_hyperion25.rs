pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyTopazVideoUpscalerHyperion25 {
    pub input: InputTopazVideoUpscalerHyperion25,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyTopazVideoUpscalerHyperion25 {
    pub fn builder() -> SubmitBodyTopazVideoUpscalerHyperion25Builder {
        <SubmitBodyTopazVideoUpscalerHyperion25Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyTopazVideoUpscalerHyperion25Builder {
    input: Option<InputTopazVideoUpscalerHyperion25>,
    webhook: Option<String>,
}

impl SubmitBodyTopazVideoUpscalerHyperion25Builder {
    pub fn input(mut self, value: InputTopazVideoUpscalerHyperion25) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyTopazVideoUpscalerHyperion25`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyTopazVideoUpscalerHyperion25Builder::input)
    pub fn build(self) -> Result<SubmitBodyTopazVideoUpscalerHyperion25, BuildError> {
        Ok(SubmitBodyTopazVideoUpscalerHyperion25 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


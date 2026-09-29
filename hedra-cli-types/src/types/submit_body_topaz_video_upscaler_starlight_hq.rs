pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyTopazVideoUpscalerStarlightHq {
    pub input: InputTopazVideoUpscalerStarlightHq,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyTopazVideoUpscalerStarlightHq {
    pub fn builder() -> SubmitBodyTopazVideoUpscalerStarlightHqBuilder {
        <SubmitBodyTopazVideoUpscalerStarlightHqBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyTopazVideoUpscalerStarlightHqBuilder {
    input: Option<InputTopazVideoUpscalerStarlightHq>,
    webhook: Option<String>,
}

impl SubmitBodyTopazVideoUpscalerStarlightHqBuilder {
    pub fn input(mut self, value: InputTopazVideoUpscalerStarlightHq) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyTopazVideoUpscalerStarlightHq`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyTopazVideoUpscalerStarlightHqBuilder::input)
    pub fn build(self) -> Result<SubmitBodyTopazVideoUpscalerStarlightHq, BuildError> {
        Ok(SubmitBodyTopazVideoUpscalerStarlightHq {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


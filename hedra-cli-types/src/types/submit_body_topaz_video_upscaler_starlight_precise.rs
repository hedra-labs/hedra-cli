pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyTopazVideoUpscalerStarlightPrecise {
    pub input: InputTopazVideoUpscalerStarlightPrecise,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyTopazVideoUpscalerStarlightPrecise {
    pub fn builder() -> SubmitBodyTopazVideoUpscalerStarlightPreciseBuilder {
        <SubmitBodyTopazVideoUpscalerStarlightPreciseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyTopazVideoUpscalerStarlightPreciseBuilder {
    input: Option<InputTopazVideoUpscalerStarlightPrecise>,
    webhook: Option<String>,
}

impl SubmitBodyTopazVideoUpscalerStarlightPreciseBuilder {
    pub fn input(mut self, value: InputTopazVideoUpscalerStarlightPrecise) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyTopazVideoUpscalerStarlightPrecise`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyTopazVideoUpscalerStarlightPreciseBuilder::input)
    pub fn build(self) -> Result<SubmitBodyTopazVideoUpscalerStarlightPrecise, BuildError> {
        Ok(SubmitBodyTopazVideoUpscalerStarlightPrecise {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


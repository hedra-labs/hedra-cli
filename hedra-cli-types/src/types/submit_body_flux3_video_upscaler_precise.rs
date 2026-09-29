pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyFlux3VideoUpscalerPrecise {
    pub input: InputFlux3VideoUpscalerPrecise,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyFlux3VideoUpscalerPrecise {
    pub fn builder() -> SubmitBodyFlux3VideoUpscalerPreciseBuilder {
        <SubmitBodyFlux3VideoUpscalerPreciseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyFlux3VideoUpscalerPreciseBuilder {
    input: Option<InputFlux3VideoUpscalerPrecise>,
    webhook: Option<String>,
}

impl SubmitBodyFlux3VideoUpscalerPreciseBuilder {
    pub fn input(mut self, value: InputFlux3VideoUpscalerPrecise) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyFlux3VideoUpscalerPrecise`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyFlux3VideoUpscalerPreciseBuilder::input)
    pub fn build(self) -> Result<SubmitBodyFlux3VideoUpscalerPrecise, BuildError> {
        Ok(SubmitBodyFlux3VideoUpscalerPrecise {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


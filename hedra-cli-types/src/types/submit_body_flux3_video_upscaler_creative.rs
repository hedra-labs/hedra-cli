pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyFlux3VideoUpscalerCreative {
    pub input: InputFlux3VideoUpscalerCreative,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyFlux3VideoUpscalerCreative {
    pub fn builder() -> SubmitBodyFlux3VideoUpscalerCreativeBuilder {
        <SubmitBodyFlux3VideoUpscalerCreativeBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyFlux3VideoUpscalerCreativeBuilder {
    input: Option<InputFlux3VideoUpscalerCreative>,
    webhook: Option<String>,
}

impl SubmitBodyFlux3VideoUpscalerCreativeBuilder {
    pub fn input(mut self, value: InputFlux3VideoUpscalerCreative) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyFlux3VideoUpscalerCreative`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyFlux3VideoUpscalerCreativeBuilder::input)
    pub fn build(self) -> Result<SubmitBodyFlux3VideoUpscalerCreative, BuildError> {
        Ok(SubmitBodyFlux3VideoUpscalerCreative {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


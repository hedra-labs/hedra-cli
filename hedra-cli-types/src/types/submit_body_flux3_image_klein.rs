pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyFlux3ImageKlein {
    pub input: InputFlux3ImageKlein,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyFlux3ImageKlein {
    pub fn builder() -> SubmitBodyFlux3ImageKleinBuilder {
        <SubmitBodyFlux3ImageKleinBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyFlux3ImageKleinBuilder {
    input: Option<InputFlux3ImageKlein>,
    webhook: Option<String>,
}

impl SubmitBodyFlux3ImageKleinBuilder {
    pub fn input(mut self, value: InputFlux3ImageKlein) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyFlux3ImageKlein`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyFlux3ImageKleinBuilder::input)
    pub fn build(self) -> Result<SubmitBodyFlux3ImageKlein, BuildError> {
        Ok(SubmitBodyFlux3ImageKlein {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


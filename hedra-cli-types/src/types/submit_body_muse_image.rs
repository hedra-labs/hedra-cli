pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyMuseImage {
    pub input: InputMuseImage,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyMuseImage {
    pub fn builder() -> SubmitBodyMuseImageBuilder {
        <SubmitBodyMuseImageBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyMuseImageBuilder {
    input: Option<InputMuseImage>,
    webhook: Option<String>,
}

impl SubmitBodyMuseImageBuilder {
    pub fn input(mut self, value: InputMuseImage) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyMuseImage`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyMuseImageBuilder::input)
    pub fn build(self) -> Result<SubmitBodyMuseImage, BuildError> {
        Ok(SubmitBodyMuseImage {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


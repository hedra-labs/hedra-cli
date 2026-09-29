pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubmitBodyHeygenPhotoAvatar4 {
    pub input: InputHeygenPhotoAvatar4,
    /// URL to receive a signed completion webhook.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,
}

impl SubmitBodyHeygenPhotoAvatar4 {
    pub fn builder() -> SubmitBodyHeygenPhotoAvatar4Builder {
        <SubmitBodyHeygenPhotoAvatar4Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SubmitBodyHeygenPhotoAvatar4Builder {
    input: Option<InputHeygenPhotoAvatar4>,
    webhook: Option<String>,
}

impl SubmitBodyHeygenPhotoAvatar4Builder {
    pub fn input(mut self, value: InputHeygenPhotoAvatar4) -> Self {
        self.input = Some(value);
        self
    }

    pub fn webhook(mut self, value: impl Into<String>) -> Self {
        self.webhook = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SubmitBodyHeygenPhotoAvatar4`].
    /// This method will fail if any of the following fields are not set:
    /// - [`input`](SubmitBodyHeygenPhotoAvatar4Builder::input)
    pub fn build(self) -> Result<SubmitBodyHeygenPhotoAvatar4, BuildError> {
        Ok(SubmitBodyHeygenPhotoAvatar4 {
            input: self.input.ok_or_else(|| BuildError::missing_field("input"))?,
            webhook: self.webhook,
        })
    }
}


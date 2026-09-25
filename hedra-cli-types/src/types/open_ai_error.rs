pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The OpenAI error object — this surface's dialect, not the v3 envelope.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct OpenAiError {
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl OpenAiError {
    pub fn builder() -> OpenAiErrorBuilder {
        <OpenAiErrorBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct OpenAiErrorBuilder {
    message: Option<String>,
    r#type: Option<String>,
    param: Option<String>,
    code: Option<String>,
}

impl OpenAiErrorBuilder {
    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    pub fn r#type(mut self, value: impl Into<String>) -> Self {
        self.r#type = Some(value.into());
        self
    }

    pub fn param(mut self, value: impl Into<String>) -> Self {
        self.param = Some(value.into());
        self
    }

    pub fn code(mut self, value: impl Into<String>) -> Self {
        self.code = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`OpenAiError`].
    /// This method will fail if any of the following fields are not set:
    /// - [`message`](OpenAiErrorBuilder::message)
    /// - [`r#type`](OpenAiErrorBuilder::r#type)
    pub fn build(self) -> Result<OpenAiError, BuildError> {
        Ok(OpenAiError {
            message: self.message.ok_or_else(|| BuildError::missing_field("message"))?,
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
            param: self.param,
            code: self.code,
        })
    }
}

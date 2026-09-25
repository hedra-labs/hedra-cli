pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct OpenAiErrorResponse {
    #[serde(default)]
    pub error: OpenAiError,
}

impl OpenAiErrorResponse {
    pub fn builder() -> OpenAiErrorResponseBuilder {
        <OpenAiErrorResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct OpenAiErrorResponseBuilder {
    error: Option<OpenAiError>,
}

impl OpenAiErrorResponseBuilder {
    pub fn error(mut self, value: OpenAiError) -> Self {
        self.error = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`OpenAiErrorResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`error`](OpenAiErrorResponseBuilder::error)
    pub fn build(self) -> Result<OpenAiErrorResponse, BuildError> {
        Ok(OpenAiErrorResponse {
            error: self.error.ok_or_else(|| BuildError::missing_field("error"))?,
        })
    }
}

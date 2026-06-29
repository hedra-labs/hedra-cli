pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GenerationError {
    /// The class of error encountered.
    pub r#type: GenerationErrorType,
    /// The error message.
    #[serde(default)]
    pub message: String,
}

impl GenerationError {
    pub fn builder() -> GenerationErrorBuilder {
        <GenerationErrorBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerationErrorBuilder {
    r#type: Option<GenerationErrorType>,
    message: Option<String>,
}

impl GenerationErrorBuilder {
    pub fn r#type(mut self, value: GenerationErrorType) -> Self {
        self.r#type = Some(value);
        self
    }

    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`GenerationError`].
    /// This method will fail if any of the following fields are not set:
    /// - [`r#type`](GenerationErrorBuilder::r#type)
    /// - [`message`](GenerationErrorBuilder::message)
    pub fn build(self) -> Result<GenerationError, BuildError> {
        Ok(GenerationError {
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
            message: self.message.ok_or_else(|| BuildError::missing_field("message"))?,
        })
    }
}

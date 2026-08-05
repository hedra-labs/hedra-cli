pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One field-level validation problem in a request body.
/// 
/// `field` is the dotted path to the offending input (e.g. `input.resolution`);
/// `allowed` lists the accepted values when the field is an enum, so a caller
/// can fix it without re-fetching the model schema.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct FieldError {
    /// Dotted path to the offending input (e.g. `input.resolution`).
    #[serde(default)]
    pub field: String,
    /// What is wrong with this field's value.
    #[serde(default)]
    pub message: String,
    /// Machine-readable hint for which constraint failed ("required", "enum", "type", …).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// The accepted values, when the field is an enum — so the request can be fixed without re-fetching the model schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed: Option<Vec<String>>,
}

impl FieldError {
    pub fn builder() -> FieldErrorBuilder {
        <FieldErrorBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct FieldErrorBuilder {
    field: Option<String>,
    message: Option<String>,
    reason: Option<String>,
    allowed: Option<Vec<String>>,
}

impl FieldErrorBuilder {
    pub fn field(mut self, value: impl Into<String>) -> Self {
        self.field = Some(value.into());
        self
    }

    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = Some(value.into());
        self
    }

    pub fn reason(mut self, value: impl Into<String>) -> Self {
        self.reason = Some(value.into());
        self
    }

    pub fn allowed(mut self, value: Vec<String>) -> Self {
        self.allowed = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`FieldError`].
    /// This method will fail if any of the following fields are not set:
    /// - [`field`](FieldErrorBuilder::field)
    /// - [`message`](FieldErrorBuilder::message)
    pub fn build(self) -> Result<FieldError, BuildError> {
        Ok(FieldError {
            field: self.field.ok_or_else(|| BuildError::missing_field("field"))?,
            message: self.message.ok_or_else(|| BuildError::missing_field("message"))?,
            reason: self.reason,
            allowed: self.allowed,
        })
    }
}

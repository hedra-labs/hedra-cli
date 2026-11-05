pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SetSandboxSecretRequest {
    /// A workspace secret's name, which is also the environment variable each instance receives it as: uppercase letters, digits and underscores, not starting with a digit, at most 63 characters, and never starting `SANDBOX_`, which the sandbox reserves.
    #[serde(default)]
    pub name: String,
    /// The secret's value: 1 byte to 16 KiB of UTF-8. It is never returned.
    #[serde(default)]
    pub value: String,
}

impl SetSandboxSecretRequest {
    pub fn builder() -> SetSandboxSecretRequestBuilder {
        <SetSandboxSecretRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SetSandboxSecretRequestBuilder {
    name: Option<String>,
    value: Option<String>,
}

impl SetSandboxSecretRequestBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SetSandboxSecretRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](SetSandboxSecretRequestBuilder::name)
    /// - [`value`](SetSandboxSecretRequestBuilder::value)
    pub fn build(self) -> Result<SetSandboxSecretRequest, BuildError> {
        Ok(SetSandboxSecretRequest {
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            value: self.value.ok_or_else(|| BuildError::missing_field("value"))?,
        })
    }
}


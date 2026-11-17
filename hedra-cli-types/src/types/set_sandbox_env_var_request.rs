pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SetSandboxEnvVarRequest {
    /// An environment variable's name: uppercase letters, digits and underscores, not starting with a digit, at most 63 characters, never starting `SANDBOX_`, and none of `HEDRA_SANDBOX_RUNNER_START`, `OTEL_RESOURCE_ATTRIBUTES`, `OTEL_SERVICE_NAME`, `TRACEPARENT`, which the sandbox reserves.
    #[serde(default)]
    pub name: String,
    /// `config` for a value any member can read back, `secret` for one that is never returned.
    pub r#type: SandboxEnvVarType,
    /// The value, UTF-8 without NUL: for `config`, 0 bytes to 64 KiB; for `secret`, 1 byte to 16 KiB. A `secret` value is never returned.
    #[serde(default)]
    pub value: String,
}

impl SetSandboxEnvVarRequest {
    pub fn builder() -> SetSandboxEnvVarRequestBuilder {
        <SetSandboxEnvVarRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SetSandboxEnvVarRequestBuilder {
    name: Option<String>,
    r#type: Option<SandboxEnvVarType>,
    value: Option<String>,
}

impl SetSandboxEnvVarRequestBuilder {
    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn r#type(mut self, value: SandboxEnvVarType) -> Self {
        self.r#type = Some(value);
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SetSandboxEnvVarRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`name`](SetSandboxEnvVarRequestBuilder::name)
    /// - [`r#type`](SetSandboxEnvVarRequestBuilder::r#type)
    /// - [`value`](SetSandboxEnvVarRequestBuilder::value)
    pub fn build(self) -> Result<SetSandboxEnvVarRequest, BuildError> {
        Ok(SetSandboxEnvVarRequest {
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
            value: self.value.ok_or_else(|| BuildError::missing_field("value"))?,
        })
    }
}


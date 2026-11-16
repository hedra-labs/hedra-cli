pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for ls
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SandboxEnvLsQueryRequest {
    /// Only variables of this type; omitted is both.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<SandboxEnvVarType>,
}

impl SandboxEnvLsQueryRequest {
    pub fn builder() -> SandboxEnvLsQueryRequestBuilder {
        <SandboxEnvLsQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SandboxEnvLsQueryRequestBuilder {
    r#type: Option<SandboxEnvVarType>,
}

impl SandboxEnvLsQueryRequestBuilder {
    pub fn r#type(mut self, value: SandboxEnvVarType) -> Self {
        self.r#type = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SandboxEnvLsQueryRequest`].
    pub fn build(self) -> Result<SandboxEnvLsQueryRequest, BuildError> {
        Ok(SandboxEnvLsQueryRequest {
            r#type: self.r#type,
        })
    }
}


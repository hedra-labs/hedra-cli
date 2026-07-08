pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct KeyRotateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grace_period_seconds: Option<i64>,
}

impl KeyRotateRequest {
    pub fn builder() -> KeyRotateRequestBuilder {
        <KeyRotateRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct KeyRotateRequestBuilder {
    grace_period_seconds: Option<i64>,
}

impl KeyRotateRequestBuilder {
    pub fn grace_period_seconds(mut self, value: i64) -> Self {
        self.grace_period_seconds = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`KeyRotateRequest`].
    pub fn build(self) -> Result<KeyRotateRequest, BuildError> {
        Ok(KeyRotateRequest {
            grace_period_seconds: self.grace_period_seconds,
        })
    }
}


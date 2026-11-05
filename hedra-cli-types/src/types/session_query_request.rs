pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for session
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SessionQueryRequest {
    /// The instance's replica number, from 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replica: Option<i64>,
}

impl SessionQueryRequest {
    pub fn builder() -> SessionQueryRequestBuilder {
        <SessionQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SessionQueryRequestBuilder {
    replica: Option<i64>,
}

impl SessionQueryRequestBuilder {
    pub fn replica(mut self, value: i64) -> Self {
        self.replica = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`SessionQueryRequest`].
    pub fn build(self) -> Result<SessionQueryRequest, BuildError> {
        Ok(SessionQueryRequest {
            replica: self.replica,
        })
    }
}


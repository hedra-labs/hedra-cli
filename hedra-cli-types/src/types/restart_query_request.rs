pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for restart
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct RestartQueryRequest {
    /// The instance's replica number, from 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replica: Option<i64>,
}

impl RestartQueryRequest {
    pub fn builder() -> RestartQueryRequestBuilder {
        <RestartQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct RestartQueryRequestBuilder {
    replica: Option<i64>,
}

impl RestartQueryRequestBuilder {
    pub fn replica(mut self, value: i64) -> Self {
        self.replica = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`RestartQueryRequest`].
    pub fn build(self) -> Result<RestartQueryRequest, BuildError> {
        Ok(RestartQueryRequest {
            replica: self.replica,
        })
    }
}


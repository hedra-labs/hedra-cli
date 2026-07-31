pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct KeyListResponse {
    #[serde(default)]
    pub data: Vec<KeySummary>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl KeyListResponse {
    pub fn builder() -> KeyListResponseBuilder {
        <KeyListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct KeyListResponseBuilder {
    data: Option<Vec<KeySummary>>,
    next_cursor: Option<String>,
}

impl KeyListResponseBuilder {
    pub fn data(mut self, value: Vec<KeySummary>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`KeyListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](KeyListResponseBuilder::data)
    pub fn build(self) -> Result<KeyListResponse, BuildError> {
        Ok(KeyListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

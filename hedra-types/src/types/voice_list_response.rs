pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct VoiceListResponse {
    /// This page of items.
    #[serde(default)]
    pub data: Vec<VoiceSummary>,
    /// Opaque cursor for the next page, or null when this response completes the list. Always present. Endpoints that serve the whole collection at once always return null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

impl VoiceListResponse {
    pub fn builder() -> VoiceListResponseBuilder {
        <VoiceListResponseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct VoiceListResponseBuilder {
    data: Option<Vec<VoiceSummary>>,
    next_cursor: Option<String>,
}

impl VoiceListResponseBuilder {
    pub fn data(mut self, value: Vec<VoiceSummary>) -> Self {
        self.data = Some(value);
        self
    }

    pub fn next_cursor(mut self, value: impl Into<String>) -> Self {
        self.next_cursor = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`VoiceListResponse`].
    /// This method will fail if any of the following fields are not set:
    /// - [`data`](VoiceListResponseBuilder::data)
    pub fn build(self) -> Result<VoiceListResponse, BuildError> {
        Ok(VoiceListResponse {
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
            next_cursor: self.next_cursor,
        })
    }
}

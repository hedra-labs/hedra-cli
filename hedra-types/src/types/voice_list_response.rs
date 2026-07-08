pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct VoiceListResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<VoiceSummary>>,
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
    pub fn build(self) -> Result<VoiceListResponse, BuildError> {
        Ok(VoiceListResponse {
            data: self.data,
            next_cursor: self.next_cursor,
        })
    }
}

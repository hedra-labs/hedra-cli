pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One voice a model accepts; `id` goes into ``input.voice_id`` on submit.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct VoiceSummary {
    #[serde(default)]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labels: Option<HashMap<String, String>>,
}

impl VoiceSummary {
    pub fn builder() -> VoiceSummaryBuilder {
        <VoiceSummaryBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct VoiceSummaryBuilder {
    id: Option<String>,
    name: Option<String>,
    preview_url: Option<String>,
    labels: Option<HashMap<String, String>>,
}

impl VoiceSummaryBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn preview_url(mut self, value: impl Into<String>) -> Self {
        self.preview_url = Some(value.into());
        self
    }

    pub fn labels(mut self, value: HashMap<String, String>) -> Self {
        self.labels = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`VoiceSummary`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](VoiceSummaryBuilder::id)
    pub fn build(self) -> Result<VoiceSummary, BuildError> {
        Ok(VoiceSummary {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            name: self.name,
            preview_url: self.preview_url,
            labels: self.labels,
        })
    }
}

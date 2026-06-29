pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct GenerateVoiceCloneRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
    /// Optional agent thread ID to associate this generation with.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_thread_id: Option<String>,
    /// Optional pre-reserved generation ID. If provided, this ID will be used instead of generating a new one. For batch operations (batch_size > 1), use generation_ids instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_id: Option<String>,
    /// Optional list of pre-reserved generation IDs for batch operations. Length must match batch_size. Mutually exclusive with generation_id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_ids: Option<Vec<String>>,
    /// The id of the Audio asset to use as the basis for the clone.
    #[serde(default)]
    pub audio_id: String,
    /// The name of the new voice. Required by ElevenLabs to create a new voice.
    #[serde(default)]
    pub name: String,
}

impl GenerateVoiceCloneRequest {
    pub fn builder() -> GenerateVoiceCloneRequestBuilder {
        <GenerateVoiceCloneRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerateVoiceCloneRequestBuilder {
    workspace_id: Option<String>,
    agent_thread_id: Option<String>,
    generation_id: Option<String>,
    generation_ids: Option<Vec<String>>,
    audio_id: Option<String>,
    name: Option<String>,
}

impl GenerateVoiceCloneRequestBuilder {
    pub fn workspace_id(mut self, value: impl Into<String>) -> Self {
        self.workspace_id = Some(value.into());
        self
    }

    pub fn agent_thread_id(mut self, value: impl Into<String>) -> Self {
        self.agent_thread_id = Some(value.into());
        self
    }

    pub fn generation_id(mut self, value: impl Into<String>) -> Self {
        self.generation_id = Some(value.into());
        self
    }

    pub fn generation_ids(mut self, value: Vec<String>) -> Self {
        self.generation_ids = Some(value);
        self
    }

    pub fn audio_id(mut self, value: impl Into<String>) -> Self {
        self.audio_id = Some(value.into());
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`GenerateVoiceCloneRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`audio_id`](GenerateVoiceCloneRequestBuilder::audio_id)
    /// - [`name`](GenerateVoiceCloneRequestBuilder::name)
    pub fn build(self) -> Result<GenerateVoiceCloneRequest, BuildError> {
        Ok(GenerateVoiceCloneRequest {
            workspace_id: self.workspace_id,
            agent_thread_id: self.agent_thread_id,
            generation_id: self.generation_id,
            generation_ids: self.generation_ids,
            audio_id: self.audio_id.ok_or_else(|| BuildError::missing_field("audio_id"))?,
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
        })
    }
}

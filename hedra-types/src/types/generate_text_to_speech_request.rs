pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GenerateTextToSpeechRequest {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<GenerateTextToSpeechRequestType>,
    /// The id of the Voice to use.
    #[serde(default)]
    pub voice_id: String,
    /// The id of the model to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    /// The text to convert to speech.
    #[serde(default)]
    pub text: String,
    /// Stability should be between 0-1, where 0 is the most stable and 1 is the most unstable. This varies the consistency between your outputs.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub stability: Option<f64>,
    /// Speed should be between 0.7 and 1.2, where 0.7 is the slowest and 1.2 is the fastest. This varies the speed of the generated speech.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub speed: Option<f64>,
    /// Language for TTS. See SupportedLanguage enum for valid values. Defaults to 'auto'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<SupportedLanguage>,
}

impl GenerateTextToSpeechRequest {
    pub fn builder() -> GenerateTextToSpeechRequestBuilder {
        <GenerateTextToSpeechRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct GenerateTextToSpeechRequestBuilder {
    workspace_id: Option<String>,
    agent_thread_id: Option<String>,
    generation_id: Option<String>,
    generation_ids: Option<Vec<String>>,
    r#type: Option<GenerateTextToSpeechRequestType>,
    voice_id: Option<String>,
    model_id: Option<String>,
    text: Option<String>,
    stability: Option<f64>,
    speed: Option<f64>,
    language: Option<SupportedLanguage>,
}

impl GenerateTextToSpeechRequestBuilder {
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

    pub fn r#type(mut self, value: GenerateTextToSpeechRequestType) -> Self {
        self.r#type = Some(value);
        self
    }

    pub fn voice_id(mut self, value: impl Into<String>) -> Self {
        self.voice_id = Some(value.into());
        self
    }

    pub fn model_id(mut self, value: impl Into<String>) -> Self {
        self.model_id = Some(value.into());
        self
    }

    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = Some(value.into());
        self
    }

    pub fn stability(mut self, value: f64) -> Self {
        self.stability = Some(value);
        self
    }

    pub fn speed(mut self, value: f64) -> Self {
        self.speed = Some(value);
        self
    }

    pub fn language(mut self, value: SupportedLanguage) -> Self {
        self.language = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`GenerateTextToSpeechRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`voice_id`](GenerateTextToSpeechRequestBuilder::voice_id)
    /// - [`text`](GenerateTextToSpeechRequestBuilder::text)
    pub fn build(self) -> Result<GenerateTextToSpeechRequest, BuildError> {
        Ok(GenerateTextToSpeechRequest {
            workspace_id: self.workspace_id,
            agent_thread_id: self.agent_thread_id,
            generation_id: self.generation_id,
            generation_ids: self.generation_ids,
            r#type: self.r#type,
            voice_id: self.voice_id.ok_or_else(|| BuildError::missing_field("voice_id"))?,
            model_id: self.model_id,
            text: self.text.ok_or_else(|| BuildError::missing_field("text"))?,
            stability: self.stability,
            speed: self.speed,
            language: self.language,
        })
    }
}

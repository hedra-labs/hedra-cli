pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum GenerateAssetResponse {
        #[serde(rename = "video")]
        #[non_exhaustive]
        Video {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            ai_model_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            start_keyframe_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            start_keyframe_url: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            end_keyframe_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            end_keyframe_url: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            audio_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            audio_generation: Option<GenerateTextToSpeechRequest>,
            #[serde(skip_serializing_if = "Option::is_none")]
            audio_start_ms: Option<i64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            reference_image_ids: Option<Vec<String>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            video_id: Option<String>,
            #[serde(default)]
            generated_video_inputs: GeneratedVideoInputs,
            #[serde(skip_serializing_if = "Option::is_none")]
            batch_size: Option<i64>,
            #[serde(default)]
            id: String,
            #[serde(default)]
            asset_id: String,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            batch_generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            batch_results: Option<Vec<BatchVideoResultItem>>,
        },

        #[serde(rename = "text_to_speech")]
        #[non_exhaustive]
        TextToSpeech {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            voice_id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            model_id: Option<String>,
            #[serde(default)]
            text: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers::option")]
            stability: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers::option")]
            speed: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            language: Option<SupportedLanguage>,
            #[serde(default)]
            asset_id: String,
            #[serde(default)]
            id: String,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        #[serde(rename = "text_to_sound")]
        #[non_exhaustive]
        TextToSound {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            ai_model_id: String,
            #[serde(default)]
            text: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            duration_seconds: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            prompt_influence: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            r#loop: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            output_format: Option<String>,
            #[serde(default)]
            id: String,
            #[serde(default)]
            asset_id: String,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        #[serde(rename = "image")]
        #[non_exhaustive]
        Image {
            #[serde(flatten)]
            data: GenerateImageResponse,
        },

        #[serde(rename = "image_to_image")]
        #[non_exhaustive]
        ImageToImage {
            #[serde(flatten)]
            data: GenerateImageResponse,
        },

        #[serde(rename = "image_upscale")]
        #[non_exhaustive]
        ImageUpscale {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            ai_model_id: String,
            #[serde(default)]
            image_id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            upscale_factor: Option<f64>,
            #[serde(default)]
            asset_id: String,
            #[serde(default)]
            id: String,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        #[serde(rename = "video_upscale")]
        #[non_exhaustive]
        VideoUpscale {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            ai_model_id: String,
            #[serde(default)]
            video_id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers::option")]
            upscale_factor: Option<f64>,
            #[serde(default)]
            asset_id: String,
            #[serde(default)]
            id: String,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        #[serde(rename = "audio_isolation")]
        #[non_exhaustive]
        AudioIsolation {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            audio_id: String,
            #[serde(default)]
            ai_model_id: String,
            #[serde(default)]
            id: String,
            #[serde(default)]
            asset_id: String,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        #[serde(rename = "speech_to_speech")]
        #[non_exhaustive]
        SpeechToSpeech {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            audio_id: String,
            #[serde(default)]
            ai_model_id: String,
            #[serde(default)]
            voice_id: String,
            #[serde(default)]
            id: String,
            #[serde(default)]
            asset_id: String,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        #[serde(rename = "voice_clone")]
        #[non_exhaustive]
        VoiceClone {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            audio_id: String,
            #[serde(default)]
            name: String,
            #[serde(default)]
            id: String,
            #[serde(default)]
            asset_id: String,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        #[serde(rename = "audio_from_video")]
        #[non_exhaustive]
        AudioFromVideo {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            audio_generation_model_id: String,
            #[serde(default)]
            video_id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            prompt: Option<String>,
            #[serde(default)]
            id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            asset_id: Option<String>,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        #[serde(rename = "video_with_audio")]
        #[non_exhaustive]
        VideoWithAudio {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            video_generation_model_id: String,
            #[serde(default)]
            video_id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            prompt: Option<String>,
            #[serde(default)]
            id: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            asset_id: Option<String>,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        #[serde(rename = "video_to_video")]
        #[non_exhaustive]
        VideoToVideo {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            ai_model_id: String,
            #[serde(default)]
            video_id: String,
            #[serde(default)]
            prompt: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            reference_image_asset_ids: Option<Vec<String>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            elements: Option<Vec<KlingO1EditElement>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            keep_audio: Option<bool>,
            #[serde(default)]
            id: String,
            #[serde(default)]
            asset_id: String,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        #[serde(rename = "motion_control")]
        #[non_exhaustive]
        MotionControl {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            agent_thread_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            generation_ids: Option<Vec<String>>,
            #[serde(default)]
            ai_model_id: String,
            #[serde(default)]
            video_id: String,
            #[serde(default)]
            start_keyframe_id: String,
            #[serde(default)]
            generated_video_inputs: GeneratedVideoInputs,
            #[serde(default)]
            id: String,
            #[serde(default)]
            asset_id: String,
            #[serde(default)]
            created_at: String,
            status: GenerationStatus,
            #[serde(default)]
            #[serde(with = "crate::core::number_serializers")]
            progress: f64,
            #[serde(skip_serializing_if = "Option::is_none")]
            eta_sec: Option<i64>,
        },

        /// Catch-all variant for unrecognized discriminant values.
        /// If the server sends a discriminant not recognized by the current SDK
        /// version, the raw payload is captured here so callers can still inspect it.
        #[serde(untagged)]
        __Unknown(serde_json::Value),
}

impl GenerateAssetResponse {
    pub fn video(generated_video_inputs: GeneratedVideoInputs, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::Video { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, ai_model_id: None, start_keyframe_id: None, start_keyframe_url: None, end_keyframe_id: None, end_keyframe_url: None, audio_id: None, audio_generation: None, audio_start_ms: None, reference_image_ids: None, video_id: None, generated_video_inputs, batch_size: None, id, asset_id, created_at, status, progress, eta_sec: None, batch_generation_id: None, batch_results: None }
    }

    pub fn text_to_speech(voice_id: String, text: String, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::TextToSpeech { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, voice_id, model_id: None, text, stability: None, speed: None, language: None, asset_id, id, created_at, status, progress, eta_sec: None }
    }

    pub fn text_to_sound(ai_model_id: String, text: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::TextToSound { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, ai_model_id, text, duration_seconds: None, prompt_influence: None, r#loop: None, output_format: None, id, asset_id, created_at, status, progress, eta_sec: None }
    }

    pub fn image(data: GenerateImageResponse) -> Self {
        Self::Image { data }
    }

    pub fn image_to_image(data: GenerateImageResponse) -> Self {
        Self::ImageToImage { data }
    }

    pub fn image_upscale(ai_model_id: String, image_id: String, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::ImageUpscale { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, ai_model_id, image_id, upscale_factor: None, asset_id, id, created_at, status, progress, eta_sec: None }
    }

    pub fn video_upscale(ai_model_id: String, video_id: String, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::VideoUpscale { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, ai_model_id, video_id, upscale_factor: None, asset_id, id, created_at, status, progress, eta_sec: None }
    }

    pub fn audio_isolation(audio_id: String, ai_model_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::AudioIsolation { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, audio_id, ai_model_id, id, asset_id, created_at, status, progress, eta_sec: None }
    }

    pub fn speech_to_speech(audio_id: String, ai_model_id: String, voice_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::SpeechToSpeech { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, audio_id, ai_model_id, voice_id, id, asset_id, created_at, status, progress, eta_sec: None }
    }

    pub fn voice_clone(audio_id: String, name: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::VoiceClone { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, audio_id, name, id, asset_id, created_at, status, progress, eta_sec: None }
    }

    pub fn audio_from_video(audio_generation_model_id: String, video_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::AudioFromVideo { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, audio_generation_model_id, video_id, prompt: None, id, asset_id: None, created_at, status, progress, eta_sec: None }
    }

    pub fn video_with_audio(video_generation_model_id: String, video_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::VideoWithAudio { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, video_generation_model_id, video_id, prompt: None, id, asset_id: None, created_at, status, progress, eta_sec: None }
    }

    pub fn video_to_video(ai_model_id: String, video_id: String, prompt: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::VideoToVideo { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, ai_model_id, video_id, prompt, reference_image_asset_ids: None, elements: None, keep_audio: None, id, asset_id, created_at, status, progress, eta_sec: None }
    }

    pub fn motion_control(ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64) -> Self {
        Self::MotionControl { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, ai_model_id, video_id, start_keyframe_id, generated_video_inputs, id, asset_id, created_at, status, progress, eta_sec: None }
    }

    pub fn video_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_ai_model_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id: Some(ai_model_id), start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_start_keyframe_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: String, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id: Some(start_keyframe_id), start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_start_keyframe_url(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: String, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url: Some(start_keyframe_url), end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_end_keyframe_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: String, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id: Some(end_keyframe_id), end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_end_keyframe_url(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: String, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url: Some(end_keyframe_url), audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_audio_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: String, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id: Some(audio_id), audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_audio_generation(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: GenerateTextToSpeechRequest, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation: Some(audio_generation), audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_audio_start_ms(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: i64, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms: Some(audio_start_ms), reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_reference_image_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Vec<String>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids: Some(reference_image_ids), video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_video_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: String, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id: Some(video_id), generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_batch_size(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: i64, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size: Some(batch_size), id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results }
    }

    pub fn video_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64, batch_generation_id: Option<String>, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec: Some(eta_sec), batch_generation_id, batch_results }
    }

    pub fn video_with_batch_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: String, batch_results: Option<Vec<BatchVideoResultItem>>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id: Some(batch_generation_id), batch_results }
    }

    pub fn video_with_batch_results(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>, batch_generation_id: Option<String>, batch_results: Vec<BatchVideoResultItem>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size, id, asset_id, created_at, status, progress, eta_sec, batch_generation_id, batch_results: Some(batch_results) }
    }

    pub fn text_to_speech_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, voice_id: String, model_id: Option<String>, text: String, stability: Option<f64>, speed: Option<f64>, language: Option<SupportedLanguage>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSpeech { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, voice_id, model_id, text, stability, speed, language, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_speech_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, voice_id: String, model_id: Option<String>, text: String, stability: Option<f64>, speed: Option<f64>, language: Option<SupportedLanguage>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSpeech { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, voice_id, model_id, text, stability, speed, language, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_speech_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, voice_id: String, model_id: Option<String>, text: String, stability: Option<f64>, speed: Option<f64>, language: Option<SupportedLanguage>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSpeech { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, voice_id, model_id, text, stability, speed, language, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_speech_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, voice_id: String, model_id: Option<String>, text: String, stability: Option<f64>, speed: Option<f64>, language: Option<SupportedLanguage>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSpeech { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), voice_id, model_id, text, stability, speed, language, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_speech_with_model_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, voice_id: String, model_id: String, text: String, stability: Option<f64>, speed: Option<f64>, language: Option<SupportedLanguage>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSpeech { workspace_id, agent_thread_id, generation_id, generation_ids, voice_id, model_id: Some(model_id), text, stability, speed, language, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_speech_with_stability(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, voice_id: String, model_id: Option<String>, text: String, stability: f64, speed: Option<f64>, language: Option<SupportedLanguage>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSpeech { workspace_id, agent_thread_id, generation_id, generation_ids, voice_id, model_id, text, stability: Some(stability), speed, language, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_speech_with_speed(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, voice_id: String, model_id: Option<String>, text: String, stability: Option<f64>, speed: f64, language: Option<SupportedLanguage>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSpeech { workspace_id, agent_thread_id, generation_id, generation_ids, voice_id, model_id, text, stability, speed: Some(speed), language, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_speech_with_language(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, voice_id: String, model_id: Option<String>, text: String, stability: Option<f64>, speed: Option<f64>, language: SupportedLanguage, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSpeech { workspace_id, agent_thread_id, generation_id, generation_ids, voice_id, model_id, text, stability, speed, language: Some(language), asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_speech_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, voice_id: String, model_id: Option<String>, text: String, stability: Option<f64>, speed: Option<f64>, language: Option<SupportedLanguage>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::TextToSpeech { workspace_id, agent_thread_id, generation_id, generation_ids, voice_id, model_id, text, stability, speed, language, asset_id, id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn text_to_sound_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, text: String, duration_seconds: Option<f64>, prompt_influence: Option<f64>, r#loop: Option<bool>, output_format: Option<String>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSound { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, ai_model_id, text, duration_seconds, prompt_influence, r#loop, output_format, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_sound_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, text: String, duration_seconds: Option<f64>, prompt_influence: Option<f64>, r#loop: Option<bool>, output_format: Option<String>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSound { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, ai_model_id, text, duration_seconds, prompt_influence, r#loop, output_format, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_sound_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, ai_model_id: String, text: String, duration_seconds: Option<f64>, prompt_influence: Option<f64>, r#loop: Option<bool>, output_format: Option<String>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSound { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, ai_model_id, text, duration_seconds, prompt_influence, r#loop, output_format, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_sound_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, ai_model_id: String, text: String, duration_seconds: Option<f64>, prompt_influence: Option<f64>, r#loop: Option<bool>, output_format: Option<String>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSound { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), ai_model_id, text, duration_seconds, prompt_influence, r#loop, output_format, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_sound_with_duration_seconds(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, text: String, duration_seconds: f64, prompt_influence: Option<f64>, r#loop: Option<bool>, output_format: Option<String>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSound { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, text, duration_seconds: Some(duration_seconds), prompt_influence, r#loop, output_format, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_sound_with_prompt_influence(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, text: String, duration_seconds: Option<f64>, prompt_influence: f64, r#loop: Option<bool>, output_format: Option<String>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSound { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, text, duration_seconds, prompt_influence: Some(prompt_influence), r#loop, output_format, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_sound_with_loop(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, text: String, duration_seconds: Option<f64>, prompt_influence: Option<f64>, r#loop: bool, output_format: Option<String>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSound { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, text, duration_seconds, prompt_influence, r#loop: Some(r#loop), output_format, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_sound_with_output_format(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, text: String, duration_seconds: Option<f64>, prompt_influence: Option<f64>, r#loop: Option<bool>, output_format: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::TextToSound { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, text, duration_seconds, prompt_influence, r#loop, output_format: Some(output_format), id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn text_to_sound_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, text: String, duration_seconds: Option<f64>, prompt_influence: Option<f64>, r#loop: Option<bool>, output_format: Option<String>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::TextToSound { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, text, duration_seconds, prompt_influence, r#loop, output_format, id, asset_id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn image_upscale_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, image_id: String, upscale_factor: Option<f64>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::ImageUpscale { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, ai_model_id, image_id, upscale_factor, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn image_upscale_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, image_id: String, upscale_factor: Option<f64>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::ImageUpscale { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, ai_model_id, image_id, upscale_factor, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn image_upscale_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, ai_model_id: String, image_id: String, upscale_factor: Option<f64>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::ImageUpscale { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, ai_model_id, image_id, upscale_factor, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn image_upscale_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, ai_model_id: String, image_id: String, upscale_factor: Option<f64>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::ImageUpscale { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), ai_model_id, image_id, upscale_factor, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn image_upscale_with_upscale_factor(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, image_id: String, upscale_factor: f64, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::ImageUpscale { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, image_id, upscale_factor: Some(upscale_factor), asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn image_upscale_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, image_id: String, upscale_factor: Option<f64>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::ImageUpscale { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, image_id, upscale_factor, asset_id, id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn video_upscale_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, upscale_factor: Option<f64>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoUpscale { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, upscale_factor, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn video_upscale_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, upscale_factor: Option<f64>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoUpscale { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, ai_model_id, video_id, upscale_factor, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn video_upscale_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, upscale_factor: Option<f64>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoUpscale { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, ai_model_id, video_id, upscale_factor, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn video_upscale_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, ai_model_id: String, video_id: String, upscale_factor: Option<f64>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoUpscale { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), ai_model_id, video_id, upscale_factor, asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn video_upscale_with_upscale_factor(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, upscale_factor: f64, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoUpscale { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, upscale_factor: Some(upscale_factor), asset_id, id, created_at, status, progress, eta_sec }
    }

    pub fn video_upscale_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, upscale_factor: Option<f64>, asset_id: String, id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::VideoUpscale { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, upscale_factor, asset_id, id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn audio_isolation_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_id: String, ai_model_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::AudioIsolation { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, audio_id, ai_model_id, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn audio_isolation_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_id: String, ai_model_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::AudioIsolation { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, audio_id, ai_model_id, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn audio_isolation_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, audio_id: String, ai_model_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::AudioIsolation { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, audio_id, ai_model_id, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn audio_isolation_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, audio_id: String, ai_model_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::AudioIsolation { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), audio_id, ai_model_id, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn audio_isolation_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_id: String, ai_model_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::AudioIsolation { workspace_id, agent_thread_id, generation_id, generation_ids, audio_id, ai_model_id, id, asset_id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn speech_to_speech_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_id: String, ai_model_id: String, voice_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::SpeechToSpeech { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, audio_id, ai_model_id, voice_id, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn speech_to_speech_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_id: String, ai_model_id: String, voice_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::SpeechToSpeech { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, audio_id, ai_model_id, voice_id, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn speech_to_speech_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, audio_id: String, ai_model_id: String, voice_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::SpeechToSpeech { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, audio_id, ai_model_id, voice_id, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn speech_to_speech_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, audio_id: String, ai_model_id: String, voice_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::SpeechToSpeech { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), audio_id, ai_model_id, voice_id, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn speech_to_speech_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_id: String, ai_model_id: String, voice_id: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::SpeechToSpeech { workspace_id, agent_thread_id, generation_id, generation_ids, audio_id, ai_model_id, voice_id, id, asset_id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn voice_clone_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_id: String, name: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VoiceClone { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, audio_id, name, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn voice_clone_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_id: String, name: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VoiceClone { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, audio_id, name, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn voice_clone_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, audio_id: String, name: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VoiceClone { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, audio_id, name, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn voice_clone_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, audio_id: String, name: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VoiceClone { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), audio_id, name, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn voice_clone_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_id: String, name: String, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::VoiceClone { workspace_id, agent_thread_id, generation_id, generation_ids, audio_id, name, id, asset_id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn audio_from_video_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::AudioFromVideo { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, audio_generation_model_id, video_id, prompt, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn audio_from_video_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::AudioFromVideo { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, audio_generation_model_id, video_id, prompt, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn audio_from_video_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, audio_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::AudioFromVideo { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, audio_generation_model_id, video_id, prompt, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn audio_from_video_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, audio_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::AudioFromVideo { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), audio_generation_model_id, video_id, prompt, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn audio_from_video_with_prompt(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_generation_model_id: String, video_id: String, prompt: String, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::AudioFromVideo { workspace_id, agent_thread_id, generation_id, generation_ids, audio_generation_model_id, video_id, prompt: Some(prompt), id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn audio_from_video_with_asset_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::AudioFromVideo { workspace_id, agent_thread_id, generation_id, generation_ids, audio_generation_model_id, video_id, prompt, id, asset_id: Some(asset_id), created_at, status, progress, eta_sec }
    }

    pub fn audio_from_video_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, audio_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::AudioFromVideo { workspace_id, agent_thread_id, generation_id, generation_ids, audio_generation_model_id, video_id, prompt, id, asset_id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn video_with_audio_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, video_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoWithAudio { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, video_generation_model_id, video_id, prompt, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_with_audio_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, video_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoWithAudio { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, video_generation_model_id, video_id, prompt, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_with_audio_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, video_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoWithAudio { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, video_generation_model_id, video_id, prompt, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_with_audio_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, video_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoWithAudio { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), video_generation_model_id, video_id, prompt, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_with_audio_with_prompt(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, video_generation_model_id: String, video_id: String, prompt: String, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoWithAudio { workspace_id, agent_thread_id, generation_id, generation_ids, video_generation_model_id, video_id, prompt: Some(prompt), id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_with_audio_with_asset_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, video_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoWithAudio { workspace_id, agent_thread_id, generation_id, generation_ids, video_generation_model_id, video_id, prompt, id, asset_id: Some(asset_id), created_at, status, progress, eta_sec }
    }

    pub fn video_with_audio_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, video_generation_model_id: String, video_id: String, prompt: Option<String>, id: String, asset_id: Option<String>, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::VideoWithAudio { workspace_id, agent_thread_id, generation_id, generation_ids, video_generation_model_id, video_id, prompt, id, asset_id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn video_to_video_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, prompt: String, reference_image_asset_ids: Option<Vec<String>>, elements: Option<Vec<KlingO1EditElement>>, keep_audio: Option<bool>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoToVideo { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, prompt, reference_image_asset_ids, elements, keep_audio, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_to_video_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, prompt: String, reference_image_asset_ids: Option<Vec<String>>, elements: Option<Vec<KlingO1EditElement>>, keep_audio: Option<bool>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoToVideo { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, ai_model_id, video_id, prompt, reference_image_asset_ids, elements, keep_audio, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_to_video_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, prompt: String, reference_image_asset_ids: Option<Vec<String>>, elements: Option<Vec<KlingO1EditElement>>, keep_audio: Option<bool>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoToVideo { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, ai_model_id, video_id, prompt, reference_image_asset_ids, elements, keep_audio, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_to_video_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, ai_model_id: String, video_id: String, prompt: String, reference_image_asset_ids: Option<Vec<String>>, elements: Option<Vec<KlingO1EditElement>>, keep_audio: Option<bool>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoToVideo { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), ai_model_id, video_id, prompt, reference_image_asset_ids, elements, keep_audio, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_to_video_with_reference_image_asset_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, prompt: String, reference_image_asset_ids: Vec<String>, elements: Option<Vec<KlingO1EditElement>>, keep_audio: Option<bool>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoToVideo { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, prompt, reference_image_asset_ids: Some(reference_image_asset_ids), elements, keep_audio, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_to_video_with_elements(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, prompt: String, reference_image_asset_ids: Option<Vec<String>>, elements: Vec<KlingO1EditElement>, keep_audio: Option<bool>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoToVideo { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, prompt, reference_image_asset_ids, elements: Some(elements), keep_audio, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_to_video_with_keep_audio(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, prompt: String, reference_image_asset_ids: Option<Vec<String>>, elements: Option<Vec<KlingO1EditElement>>, keep_audio: bool, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::VideoToVideo { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, prompt, reference_image_asset_ids, elements, keep_audio: Some(keep_audio), id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn video_to_video_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, prompt: String, reference_image_asset_ids: Option<Vec<String>>, elements: Option<Vec<KlingO1EditElement>>, keep_audio: Option<bool>, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::VideoToVideo { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, prompt, reference_image_asset_ids, elements, keep_audio, id, asset_id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn motion_control_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::MotionControl { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, start_keyframe_id, generated_video_inputs, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn motion_control_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::MotionControl { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, ai_model_id, video_id, start_keyframe_id, generated_video_inputs, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn motion_control_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::MotionControl { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, ai_model_id, video_id, start_keyframe_id, generated_video_inputs, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn motion_control_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: Option<i64>) -> Self {
        Self::MotionControl { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), ai_model_id, video_id, start_keyframe_id, generated_video_inputs, id, asset_id, created_at, status, progress, eta_sec }
    }

    pub fn motion_control_with_eta_sec(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs, id: String, asset_id: String, created_at: String, status: GenerationStatus, progress: f64, eta_sec: i64) -> Self {
        Self::MotionControl { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, start_keyframe_id, generated_video_inputs, id, asset_id, created_at, status, progress, eta_sec: Some(eta_sec) }
    }

    pub fn unknown(value: serde_json::Value) -> Self {
        Self::__Unknown(value)
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum GenerateAssetRequest {
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
        },

        #[serde(rename = "text_to_speech")]
        #[non_exhaustive]
        TextToSpeech {
            #[serde(flatten)]
            data: GenerateTextToSpeechRequest,
        },

        #[serde(rename = "text_to_sound")]
        #[non_exhaustive]
        TextToSound {
            #[serde(flatten)]
            data: GenerateTextToSoundRequest,
        },

        #[serde(rename = "image")]
        #[non_exhaustive]
        Image {
            #[serde(flatten)]
            data: GenerateImageRequest,
        },

        #[serde(rename = "image_to_image")]
        #[non_exhaustive]
        ImageToImage {
            #[serde(flatten)]
            data: GenerateImageRequest,
        },

        #[serde(rename = "image_upscale")]
        #[non_exhaustive]
        ImageUpscale {
            #[serde(flatten)]
            data: GenerateImageUpscaleRequest,
        },

        #[serde(rename = "video_upscale")]
        #[non_exhaustive]
        VideoUpscale {
            #[serde(flatten)]
            data: GenerateVideoUpscaleRequest,
        },

        #[serde(rename = "audio_isolation")]
        #[non_exhaustive]
        AudioIsolation {
            #[serde(flatten)]
            data: GenerateIsolatedAudioRequest,
        },

        #[serde(rename = "speech_to_speech")]
        #[non_exhaustive]
        SpeechToSpeech {
            #[serde(flatten)]
            data: GenerateSpeechToSpeechRequest,
        },

        #[serde(rename = "voice_clone")]
        #[non_exhaustive]
        VoiceClone {
            #[serde(flatten)]
            data: GenerateVoiceCloneRequest,
        },

        #[serde(rename = "audio_from_video")]
        #[non_exhaustive]
        AudioFromVideo {
            #[serde(flatten)]
            data: GenerateAudioFromVideoRequest,
        },

        #[serde(rename = "video_with_audio")]
        #[non_exhaustive]
        VideoWithAudio {
            #[serde(flatten)]
            data: GenerateVideoWithAudioRequest,
        },

        #[serde(rename = "video_to_video")]
        #[non_exhaustive]
        VideoToVideo {
            #[serde(flatten)]
            data: GenerateVideoToVideoRequest,
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
        },

        /// Catch-all variant for unrecognized discriminant values.
        /// If the server sends a discriminant not recognized by the current SDK
        /// version, the raw payload is captured here so callers can still inspect it.
        #[serde(untagged)]
        __Unknown(serde_json::Value),
}

impl GenerateAssetRequest {
    pub fn video(generated_video_inputs: GeneratedVideoInputs) -> Self {
        Self::Video { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, ai_model_id: None, start_keyframe_id: None, start_keyframe_url: None, end_keyframe_id: None, end_keyframe_url: None, audio_id: None, audio_generation: None, audio_start_ms: None, reference_image_ids: None, video_id: None, generated_video_inputs, batch_size: None }
    }

    pub fn text_to_speech(data: GenerateTextToSpeechRequest) -> Self {
        Self::TextToSpeech { data }
    }

    pub fn text_to_sound(data: GenerateTextToSoundRequest) -> Self {
        Self::TextToSound { data }
    }

    pub fn image(data: GenerateImageRequest) -> Self {
        Self::Image { data }
    }

    pub fn image_to_image(data: GenerateImageRequest) -> Self {
        Self::ImageToImage { data }
    }

    pub fn image_upscale(data: GenerateImageUpscaleRequest) -> Self {
        Self::ImageUpscale { data }
    }

    pub fn video_upscale(data: GenerateVideoUpscaleRequest) -> Self {
        Self::VideoUpscale { data }
    }

    pub fn audio_isolation(data: GenerateIsolatedAudioRequest) -> Self {
        Self::AudioIsolation { data }
    }

    pub fn speech_to_speech(data: GenerateSpeechToSpeechRequest) -> Self {
        Self::SpeechToSpeech { data }
    }

    pub fn voice_clone(data: GenerateVoiceCloneRequest) -> Self {
        Self::VoiceClone { data }
    }

    pub fn audio_from_video(data: GenerateAudioFromVideoRequest) -> Self {
        Self::AudioFromVideo { data }
    }

    pub fn video_with_audio(data: GenerateVideoWithAudioRequest) -> Self {
        Self::VideoWithAudio { data }
    }

    pub fn video_to_video(data: GenerateVideoToVideoRequest) -> Self {
        Self::VideoToVideo { data }
    }

    pub fn motion_control(ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs) -> Self {
        Self::MotionControl { workspace_id: None, agent_thread_id: None, generation_id: None, generation_ids: None, ai_model_id, video_id, start_keyframe_id, generated_video_inputs }
    }

    pub fn video_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_ai_model_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id: Some(ai_model_id), start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_start_keyframe_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: String, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id: Some(start_keyframe_id), start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_start_keyframe_url(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: String, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url: Some(start_keyframe_url), end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_end_keyframe_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: String, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id: Some(end_keyframe_id), end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_end_keyframe_url(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: String, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url: Some(end_keyframe_url), audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_audio_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: String, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id: Some(audio_id), audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_audio_generation(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: GenerateTextToSpeechRequest, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation: Some(audio_generation), audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_audio_start_ms(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: i64, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms: Some(audio_start_ms), reference_image_ids, video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_reference_image_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Vec<String>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids: Some(reference_image_ids), video_id, generated_video_inputs, batch_size }
    }

    pub fn video_with_video_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: String, generated_video_inputs: GeneratedVideoInputs, batch_size: Option<i64>) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id: Some(video_id), generated_video_inputs, batch_size }
    }

    pub fn video_with_batch_size(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: Option<String>, start_keyframe_id: Option<String>, start_keyframe_url: Option<String>, end_keyframe_id: Option<String>, end_keyframe_url: Option<String>, audio_id: Option<String>, audio_generation: Option<GenerateTextToSpeechRequest>, audio_start_ms: Option<i64>, reference_image_ids: Option<Vec<String>>, video_id: Option<String>, generated_video_inputs: GeneratedVideoInputs, batch_size: i64) -> Self {
        Self::Video { workspace_id, agent_thread_id, generation_id, generation_ids, ai_model_id, start_keyframe_id, start_keyframe_url, end_keyframe_id, end_keyframe_url, audio_id, audio_generation, audio_start_ms, reference_image_ids, video_id, generated_video_inputs, batch_size: Some(batch_size) }
    }

    pub fn motion_control_with_workspace_id(workspace_id: String, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs) -> Self {
        Self::MotionControl { workspace_id: Some(workspace_id), agent_thread_id, generation_id, generation_ids, ai_model_id, video_id, start_keyframe_id, generated_video_inputs }
    }

    pub fn motion_control_with_agent_thread_id(workspace_id: Option<String>, agent_thread_id: String, generation_id: Option<String>, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs) -> Self {
        Self::MotionControl { workspace_id, agent_thread_id: Some(agent_thread_id), generation_id, generation_ids, ai_model_id, video_id, start_keyframe_id, generated_video_inputs }
    }

    pub fn motion_control_with_generation_id(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: String, generation_ids: Option<Vec<String>>, ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs) -> Self {
        Self::MotionControl { workspace_id, agent_thread_id, generation_id: Some(generation_id), generation_ids, ai_model_id, video_id, start_keyframe_id, generated_video_inputs }
    }

    pub fn motion_control_with_generation_ids(workspace_id: Option<String>, agent_thread_id: Option<String>, generation_id: Option<String>, generation_ids: Vec<String>, ai_model_id: String, video_id: String, start_keyframe_id: String, generated_video_inputs: GeneratedVideoInputs) -> Self {
        Self::MotionControl { workspace_id, agent_thread_id, generation_id, generation_ids: Some(generation_ids), ai_model_id, video_id, start_keyframe_id, generated_video_inputs }
    }

    pub fn unknown(value: serde_json::Value) -> Self {
        Self::__Unknown(value)
    }
}

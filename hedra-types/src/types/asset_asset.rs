pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum AssetAsset {
        #[serde(rename = "generated_audio")]
        #[non_exhaustive]
        GeneratedAudio {
            #[serde(default)]
            duration_ms: i64,
            #[serde(default)]
            url: String,
            #[serde(default)]
            generated_audio_inputs: GeneratedAudioInputs,
            #[serde(skip_serializing_if = "Option::is_none")]
            transcriptions: Option<Vec<HashMap<String, serde_json::Value>>>,
        },

        #[serde(rename = "generated_image")]
        #[non_exhaustive]
        GeneratedImage {
            #[serde(default)]
            width: i64,
            #[serde(default)]
            height: i64,
            #[serde(default)]
            url: String,
            #[serde(default)]
            generated_image_inputs: GeneratedImageInputs,
        },

        #[serde(rename = "generated_video")]
        #[non_exhaustive]
        GeneratedVideo {
            #[serde(default)]
            width: i64,
            #[serde(default)]
            height: i64,
            #[serde(skip_serializing_if = "Option::is_none")]
            url: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            download_url: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            streaming_url: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            poster_url: Option<String>,
            #[serde(default)]
            duration_ms: i64,
            #[serde(default)]
            preview_url: String,
            #[serde(default)]
            generated_video_inputs: GeneratedVideoInputs,
            #[serde(skip_serializing_if = "Option::is_none")]
            keyframe_start: Option<Box<Asset>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            keyframe_end: Option<Box<Asset>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            audio: Option<Box<Asset>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            reference_image_ids: Option<Vec<String>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            source_video_id: Option<String>,
        },

        #[serde(rename = "uploaded_audio")]
        #[non_exhaustive]
        UploadedAudio {
            #[serde(default)]
            duration_ms: i64,
            #[serde(default)]
            url: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            transcriptions: Option<Vec<HashMap<String, serde_json::Value>>>,
        },

        #[serde(rename = "uploaded_image")]
        #[non_exhaustive]
        UploadedImage {
            #[serde(default)]
            width: i64,
            #[serde(default)]
            height: i64,
            #[serde(default)]
            url: String,
        },

        #[serde(rename = "uploaded_video")]
        #[non_exhaustive]
        UploadedVideo {
            #[serde(default)]
            width: i64,
            #[serde(default)]
            height: i64,
            #[serde(default)]
            duration_ms: i64,
            #[serde(default)]
            url: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            download_url: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            streaming_url: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            poster_url: Option<String>,
        },

        #[serde(rename = "voice")]
        #[non_exhaustive]
        Voice {
            #[serde(skip_serializing_if = "Option::is_none")]
            external_id: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            labels: Option<Vec<VoiceLabel>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            preview_url: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            source: Option<String>,
        },

        /// Catch-all variant for unrecognized discriminant values.
        /// If the server sends a discriminant not recognized by the current SDK
        /// version, the raw payload is captured here so callers can still inspect it.
        #[serde(untagged)]
        __Unknown(serde_json::Value),
}

impl AssetAsset {
    pub fn generated_audio(duration_ms: i64, url: String, generated_audio_inputs: GeneratedAudioInputs) -> Self {
        Self::GeneratedAudio { duration_ms, url, generated_audio_inputs, transcriptions: None }
    }

    pub fn generated_image(width: i64, height: i64, url: String, generated_image_inputs: GeneratedImageInputs) -> Self {
        Self::GeneratedImage { width, height, url, generated_image_inputs }
    }

    pub fn generated_video(width: i64, height: i64, duration_ms: i64, preview_url: String, generated_video_inputs: GeneratedVideoInputs) -> Self {
        Self::GeneratedVideo { width, height, url: None, download_url: None, streaming_url: None, poster_url: None, duration_ms, preview_url, generated_video_inputs, keyframe_start: None, keyframe_end: None, audio: None, reference_image_ids: None, source_video_id: None }
    }

    pub fn uploaded_audio(duration_ms: i64, url: String) -> Self {
        Self::UploadedAudio { duration_ms, url, transcriptions: None }
    }

    pub fn uploaded_image(width: i64, height: i64, url: String) -> Self {
        Self::UploadedImage { width, height, url }
    }

    pub fn uploaded_video(width: i64, height: i64, duration_ms: i64, url: String) -> Self {
        Self::UploadedVideo { width, height, duration_ms, url, download_url: None, streaming_url: None, poster_url: None }
    }

    pub fn voice() -> Self {
        Self::Voice { external_id: None, labels: None, preview_url: None, source: None }
    }

    pub fn generated_audio_with_transcriptions(duration_ms: i64, url: String, generated_audio_inputs: GeneratedAudioInputs, transcriptions: Vec<HashMap<String, serde_json::Value>>) -> Self {
        Self::GeneratedAudio { duration_ms, url, generated_audio_inputs, transcriptions: Some(transcriptions) }
    }

    pub fn generated_video_with_url(width: i64, height: i64, url: String, download_url: Option<String>, streaming_url: Option<String>, poster_url: Option<String>, duration_ms: i64, preview_url: String, generated_video_inputs: GeneratedVideoInputs, keyframe_start: Option<Box<Asset>>, keyframe_end: Option<Box<Asset>>, audio: Option<Box<Asset>>, reference_image_ids: Option<Vec<String>>, source_video_id: Option<String>) -> Self {
        Self::GeneratedVideo { width, height, url: Some(url), download_url, streaming_url, poster_url, duration_ms, preview_url, generated_video_inputs, keyframe_start, keyframe_end, audio, reference_image_ids, source_video_id }
    }

    pub fn generated_video_with_download_url(width: i64, height: i64, url: Option<String>, download_url: String, streaming_url: Option<String>, poster_url: Option<String>, duration_ms: i64, preview_url: String, generated_video_inputs: GeneratedVideoInputs, keyframe_start: Option<Box<Asset>>, keyframe_end: Option<Box<Asset>>, audio: Option<Box<Asset>>, reference_image_ids: Option<Vec<String>>, source_video_id: Option<String>) -> Self {
        Self::GeneratedVideo { width, height, url, download_url: Some(download_url), streaming_url, poster_url, duration_ms, preview_url, generated_video_inputs, keyframe_start, keyframe_end, audio, reference_image_ids, source_video_id }
    }

    pub fn generated_video_with_streaming_url(width: i64, height: i64, url: Option<String>, download_url: Option<String>, streaming_url: String, poster_url: Option<String>, duration_ms: i64, preview_url: String, generated_video_inputs: GeneratedVideoInputs, keyframe_start: Option<Box<Asset>>, keyframe_end: Option<Box<Asset>>, audio: Option<Box<Asset>>, reference_image_ids: Option<Vec<String>>, source_video_id: Option<String>) -> Self {
        Self::GeneratedVideo { width, height, url, download_url, streaming_url: Some(streaming_url), poster_url, duration_ms, preview_url, generated_video_inputs, keyframe_start, keyframe_end, audio, reference_image_ids, source_video_id }
    }

    pub fn generated_video_with_poster_url(width: i64, height: i64, url: Option<String>, download_url: Option<String>, streaming_url: Option<String>, poster_url: String, duration_ms: i64, preview_url: String, generated_video_inputs: GeneratedVideoInputs, keyframe_start: Option<Box<Asset>>, keyframe_end: Option<Box<Asset>>, audio: Option<Box<Asset>>, reference_image_ids: Option<Vec<String>>, source_video_id: Option<String>) -> Self {
        Self::GeneratedVideo { width, height, url, download_url, streaming_url, poster_url: Some(poster_url), duration_ms, preview_url, generated_video_inputs, keyframe_start, keyframe_end, audio, reference_image_ids, source_video_id }
    }

    pub fn generated_video_with_keyframe_start(width: i64, height: i64, url: Option<String>, download_url: Option<String>, streaming_url: Option<String>, poster_url: Option<String>, duration_ms: i64, preview_url: String, generated_video_inputs: GeneratedVideoInputs, keyframe_start: Box<Asset>, keyframe_end: Option<Box<Asset>>, audio: Option<Box<Asset>>, reference_image_ids: Option<Vec<String>>, source_video_id: Option<String>) -> Self {
        Self::GeneratedVideo { width, height, url, download_url, streaming_url, poster_url, duration_ms, preview_url, generated_video_inputs, keyframe_start: Some(keyframe_start), keyframe_end, audio, reference_image_ids, source_video_id }
    }

    pub fn generated_video_with_keyframe_end(width: i64, height: i64, url: Option<String>, download_url: Option<String>, streaming_url: Option<String>, poster_url: Option<String>, duration_ms: i64, preview_url: String, generated_video_inputs: GeneratedVideoInputs, keyframe_start: Option<Box<Asset>>, keyframe_end: Box<Asset>, audio: Option<Box<Asset>>, reference_image_ids: Option<Vec<String>>, source_video_id: Option<String>) -> Self {
        Self::GeneratedVideo { width, height, url, download_url, streaming_url, poster_url, duration_ms, preview_url, generated_video_inputs, keyframe_start, keyframe_end: Some(keyframe_end), audio, reference_image_ids, source_video_id }
    }

    pub fn generated_video_with_audio(width: i64, height: i64, url: Option<String>, download_url: Option<String>, streaming_url: Option<String>, poster_url: Option<String>, duration_ms: i64, preview_url: String, generated_video_inputs: GeneratedVideoInputs, keyframe_start: Option<Box<Asset>>, keyframe_end: Option<Box<Asset>>, audio: Box<Asset>, reference_image_ids: Option<Vec<String>>, source_video_id: Option<String>) -> Self {
        Self::GeneratedVideo { width, height, url, download_url, streaming_url, poster_url, duration_ms, preview_url, generated_video_inputs, keyframe_start, keyframe_end, audio: Some(audio), reference_image_ids, source_video_id }
    }

    pub fn generated_video_with_reference_image_ids(width: i64, height: i64, url: Option<String>, download_url: Option<String>, streaming_url: Option<String>, poster_url: Option<String>, duration_ms: i64, preview_url: String, generated_video_inputs: GeneratedVideoInputs, keyframe_start: Option<Box<Asset>>, keyframe_end: Option<Box<Asset>>, audio: Option<Box<Asset>>, reference_image_ids: Vec<String>, source_video_id: Option<String>) -> Self {
        Self::GeneratedVideo { width, height, url, download_url, streaming_url, poster_url, duration_ms, preview_url, generated_video_inputs, keyframe_start, keyframe_end, audio, reference_image_ids: Some(reference_image_ids), source_video_id }
    }

    pub fn generated_video_with_source_video_id(width: i64, height: i64, url: Option<String>, download_url: Option<String>, streaming_url: Option<String>, poster_url: Option<String>, duration_ms: i64, preview_url: String, generated_video_inputs: GeneratedVideoInputs, keyframe_start: Option<Box<Asset>>, keyframe_end: Option<Box<Asset>>, audio: Option<Box<Asset>>, reference_image_ids: Option<Vec<String>>, source_video_id: String) -> Self {
        Self::GeneratedVideo { width, height, url, download_url, streaming_url, poster_url, duration_ms, preview_url, generated_video_inputs, keyframe_start, keyframe_end, audio, reference_image_ids, source_video_id: Some(source_video_id) }
    }

    pub fn uploaded_audio_with_transcriptions(duration_ms: i64, url: String, transcriptions: Vec<HashMap<String, serde_json::Value>>) -> Self {
        Self::UploadedAudio { duration_ms, url, transcriptions: Some(transcriptions) }
    }

    pub fn uploaded_video_with_download_url(width: i64, height: i64, duration_ms: i64, url: String, download_url: String, streaming_url: Option<String>, poster_url: Option<String>) -> Self {
        Self::UploadedVideo { width, height, duration_ms, url, download_url: Some(download_url), streaming_url, poster_url }
    }

    pub fn uploaded_video_with_streaming_url(width: i64, height: i64, duration_ms: i64, url: String, download_url: Option<String>, streaming_url: String, poster_url: Option<String>) -> Self {
        Self::UploadedVideo { width, height, duration_ms, url, download_url, streaming_url: Some(streaming_url), poster_url }
    }

    pub fn uploaded_video_with_poster_url(width: i64, height: i64, duration_ms: i64, url: String, download_url: Option<String>, streaming_url: Option<String>, poster_url: String) -> Self {
        Self::UploadedVideo { width, height, duration_ms, url, download_url, streaming_url, poster_url: Some(poster_url) }
    }

    pub fn voice_with_external_id(external_id: String, labels: Option<Vec<VoiceLabel>>, preview_url: Option<String>, source: Option<String>) -> Self {
        Self::Voice { external_id: Some(external_id), labels, preview_url, source }
    }

    pub fn voice_with_labels(external_id: Option<String>, labels: Vec<VoiceLabel>, preview_url: Option<String>, source: Option<String>) -> Self {
        Self::Voice { external_id, labels: Some(labels), preview_url, source }
    }

    pub fn voice_with_preview_url(external_id: Option<String>, labels: Option<Vec<VoiceLabel>>, preview_url: String, source: Option<String>) -> Self {
        Self::Voice { external_id, labels, preview_url: Some(preview_url), source }
    }

    pub fn voice_with_source(external_id: Option<String>, labels: Option<Vec<VoiceLabel>>, preview_url: Option<String>, source: String) -> Self {
        Self::Voice { external_id, labels, preview_url, source: Some(source) }
    }

    pub fn unknown(value: serde_json::Value) -> Self {
        Self::__Unknown(value)
    }
}

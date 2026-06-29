pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Generation type enum
/// 
/// NOTE: this enum is used to determine the type of generation and is used to determine
/// the type of asset that will be generated.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenerationType {
    Image,
    Video,
    TextToSpeech,
    SpeechToSpeech,
    VoiceClone,
    AudioIsolation,
    VideoStitching,
    VideoUpscale,
    VideoToVideo,
    ImageUpscale,
    AgentResponse,
    AudioFromVideo,
    TextToSound,
    AssetsToImageTextPrompt,
    AssetsToAudioTextPrompt,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for GenerationType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Image => serializer.serialize_str("image"),
            Self::Video => serializer.serialize_str("video"),
            Self::TextToSpeech => serializer.serialize_str("text_to_speech"),
            Self::SpeechToSpeech => serializer.serialize_str("speech_to_speech"),
            Self::VoiceClone => serializer.serialize_str("voice_clone"),
            Self::AudioIsolation => serializer.serialize_str("audio_isolation"),
            Self::VideoStitching => serializer.serialize_str("video_stitching"),
            Self::VideoUpscale => serializer.serialize_str("video_upscale"),
            Self::VideoToVideo => serializer.serialize_str("video_to_video"),
            Self::ImageUpscale => serializer.serialize_str("image_upscale"),
            Self::AgentResponse => serializer.serialize_str("agent_response"),
            Self::AudioFromVideo => serializer.serialize_str("audio_from_video"),
            Self::TextToSound => serializer.serialize_str("text_to_sound"),
            Self::AssetsToImageTextPrompt => serializer.serialize_str("assets_to_image_text_prompt"),
            Self::AssetsToAudioTextPrompt => serializer.serialize_str("assets_to_audio_text_prompt"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for GenerationType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "image" => Ok(Self::Image),
            "video" => Ok(Self::Video),
            "text_to_speech" => Ok(Self::TextToSpeech),
            "speech_to_speech" => Ok(Self::SpeechToSpeech),
            "voice_clone" => Ok(Self::VoiceClone),
            "audio_isolation" => Ok(Self::AudioIsolation),
            "video_stitching" => Ok(Self::VideoStitching),
            "video_upscale" => Ok(Self::VideoUpscale),
            "video_to_video" => Ok(Self::VideoToVideo),
            "image_upscale" => Ok(Self::ImageUpscale),
            "agent_response" => Ok(Self::AgentResponse),
            "audio_from_video" => Ok(Self::AudioFromVideo),
            "text_to_sound" => Ok(Self::TextToSound),
            "assets_to_image_text_prompt" => Ok(Self::AssetsToImageTextPrompt),
            "assets_to_audio_text_prompt" => Ok(Self::AssetsToAudioTextPrompt),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for GenerationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image => write!(f, "image"),
            Self::Video => write!(f, "video"),
            Self::TextToSpeech => write!(f, "text_to_speech"),
            Self::SpeechToSpeech => write!(f, "speech_to_speech"),
            Self::VoiceClone => write!(f, "voice_clone"),
            Self::AudioIsolation => write!(f, "audio_isolation"),
            Self::VideoStitching => write!(f, "video_stitching"),
            Self::VideoUpscale => write!(f, "video_upscale"),
            Self::VideoToVideo => write!(f, "video_to_video"),
            Self::ImageUpscale => write!(f, "image_upscale"),
            Self::AgentResponse => write!(f, "agent_response"),
            Self::AudioFromVideo => write!(f, "audio_from_video"),
            Self::TextToSound => write!(f, "text_to_sound"),
            Self::AssetsToImageTextPrompt => write!(f, "assets_to_image_text_prompt"),
            Self::AssetsToAudioTextPrompt => write!(f, "assets_to_audio_text_prompt"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

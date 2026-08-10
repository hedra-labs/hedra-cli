pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Whether the output character's orientation follows the reference video ('video' — better for complex motion) or the character image ('image' — better for camera movement). Also caps the source video: 30s for 'video', 10s for 'image'.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputKling26MotionControlCharacterOrientation {
    Image,
    Video,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputKling26MotionControlCharacterOrientation {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Image => serializer.serialize_str("image"),
            Self::Video => serializer.serialize_str("video"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputKling26MotionControlCharacterOrientation {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "image" => Ok(Self::Image),
            "video" => Ok(Self::Video),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputKling26MotionControlCharacterOrientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Image => write!(f, "image"),
            Self::Video => write!(f, "video"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

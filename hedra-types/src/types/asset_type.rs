pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AssetType {
    Text,
    Image,
    Audio,
    Video,
    Voice,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for AssetType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Text => serializer.serialize_str("text"),
            Self::Image => serializer.serialize_str("image"),
            Self::Audio => serializer.serialize_str("audio"),
            Self::Video => serializer.serialize_str("video"),
            Self::Voice => serializer.serialize_str("voice"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for AssetType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "text" => Ok(Self::Text),
            "image" => Ok(Self::Image),
            "audio" => Ok(Self::Audio),
            "video" => Ok(Self::Video),
            "voice" => Ok(Self::Voice),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for AssetType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Text => write!(f, "text"),
            Self::Image => write!(f, "image"),
            Self::Audio => write!(f, "audio"),
            Self::Video => write!(f, "video"),
            Self::Voice => write!(f, "voice"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

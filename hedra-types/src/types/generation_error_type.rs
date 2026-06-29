pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Generation error type
/// 
/// NOTE: The backend will try to map any error happening during generation to one of these classes.
/// If it fails to do so it will map onto UNKNOWN.
/// 
/// NOTE: The semantic meaning of these errors types is roughly mapped from the semantic of the gRPC
/// status codes: see https://grpc.io/docs/guides/status-codes/ for details. Not all the gGRPC
/// status codes are mapped here (not needed at the moment).
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GenerationErrorType {
    Unknown,
    ModerationFailed,
    MissingCredits,
    ResourceExhausted,
    InvalidArgument,
    Unavailable,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for GenerationErrorType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Unknown => serializer.serialize_str("UNKNOWN"),
            Self::ModerationFailed => serializer.serialize_str("MODERATION_FAILED"),
            Self::MissingCredits => serializer.serialize_str("MISSING_CREDITS"),
            Self::ResourceExhausted => serializer.serialize_str("RESOURCE_EXHAUSTED"),
            Self::InvalidArgument => serializer.serialize_str("INVALID_ARGUMENT"),
            Self::Unavailable => serializer.serialize_str("UNAVAILABLE"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for GenerationErrorType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "UNKNOWN" => Ok(Self::Unknown),
            "MODERATION_FAILED" => Ok(Self::ModerationFailed),
            "MISSING_CREDITS" => Ok(Self::MissingCredits),
            "RESOURCE_EXHAUSTED" => Ok(Self::ResourceExhausted),
            "INVALID_ARGUMENT" => Ok(Self::InvalidArgument),
            "UNAVAILABLE" => Ok(Self::Unavailable),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for GenerationErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "UNKNOWN"),
            Self::ModerationFailed => write!(f, "MODERATION_FAILED"),
            Self::MissingCredits => write!(f, "MISSING_CREDITS"),
            Self::ResourceExhausted => write!(f, "RESOURCE_EXHAUSTED"),
            Self::InvalidArgument => write!(f, "INVALID_ARGUMENT"),
            Self::Unavailable => write!(f, "UNAVAILABLE"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

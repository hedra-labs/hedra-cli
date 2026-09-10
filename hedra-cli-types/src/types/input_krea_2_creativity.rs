pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// How loosely the model may read the prompt. 'raw' stays closest to it; 'high' drifts furthest for a more inventive result. Omit for Krea's own default ('medium').
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputKrea2Creativity {
    Raw,
    Low,
    Medium,
    High,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputKrea2Creativity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Raw => serializer.serialize_str("raw"),
            Self::Low => serializer.serialize_str("low"),
            Self::Medium => serializer.serialize_str("medium"),
            Self::High => serializer.serialize_str("high"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputKrea2Creativity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "raw" => Ok(Self::Raw),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputKrea2Creativity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Raw => write!(f, "raw"),
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

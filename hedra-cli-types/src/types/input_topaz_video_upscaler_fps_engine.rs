pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Interpolate the output to 60fps with this engine. Omit to keep the source rate; a source already at 60fps or faster keeps its own. 'high' doubles the charge.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputTopazVideoUpscalerFpsEngine {
    Standard,
    High,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputTopazVideoUpscalerFpsEngine {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Standard => serializer.serialize_str("standard"),
            Self::High => serializer.serialize_str("high"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputTopazVideoUpscalerFpsEngine {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "standard" => Ok(Self::Standard),
            "high" => Ok(Self::High),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputTopazVideoUpscalerFpsEngine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::High => write!(f, "high"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Where the enhancement parameters come from: 'standard' uses Hedra's per-resolution presets, 'auto' lets Topaz estimate them from the source.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputTopazVideoUpscalerTuning {
    Standard,
    Auto,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputTopazVideoUpscalerTuning {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Standard => serializer.serialize_str("standard"),
            Self::Auto => serializer.serialize_str("auto"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputTopazVideoUpscalerTuning {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "standard" => Ok(Self::Standard),
            "auto" => Ok(Self::Auto),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputTopazVideoUpscalerTuning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Auto => write!(f, "auto"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

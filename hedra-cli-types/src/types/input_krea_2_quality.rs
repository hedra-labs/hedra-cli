pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at. `medium` — the smaller model, strongest on illustration, anime and painting. `large` — the larger model, for photorealism and the effects that sell it — motion blur, grain, low dynamic range.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputKrea2Quality {
    Medium,
    Large,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputKrea2Quality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Medium => serializer.serialize_str("medium"),
            Self::Large => serializer.serialize_str("large"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputKrea2Quality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "medium" => Ok(Self::Medium),
            "large" => Ok(Self::Large),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputKrea2Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Medium => write!(f, "medium"),
            Self::Large => write!(f, "large"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

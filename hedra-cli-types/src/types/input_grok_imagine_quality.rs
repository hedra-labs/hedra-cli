pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at. `standard` — the base tier, at a flat rate whatever the output size. `quality` — xAI's higher-fidelity tier, and the only one offering 2k.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputGrokImagineQuality {
    Standard,
    Quality,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputGrokImagineQuality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Standard => serializer.serialize_str("standard"),
            Self::Quality => serializer.serialize_str("quality"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputGrokImagineQuality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "standard" => Ok(Self::Standard),
            "quality" => Ok(Self::Quality),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputGrokImagineQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Quality => write!(f, "quality"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

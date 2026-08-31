pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at. `standard` — the base tier, offering 2K and 4K and the only one with a reference mode. `max` — a post-trained variant at half the price, 480p and 768p only.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputMinimaxH3Quality {
    Standard,
    Max,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputMinimaxH3Quality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Standard => serializer.serialize_str("standard"),
            Self::Max => serializer.serialize_str("max"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputMinimaxH3Quality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "standard" => Ok(Self::Standard),
            "max" => Ok(Self::Max),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputMinimaxH3Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Max => write!(f, "max"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

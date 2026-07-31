pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputHidreamO1ImageQuality {
    Standard,
    Dev,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputHidreamO1ImageQuality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Standard => serializer.serialize_str("standard"),
            Self::Dev => serializer.serialize_str("dev"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputHidreamO1ImageQuality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "standard" => Ok(Self::Standard),
            "dev" => Ok(Self::Dev),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputHidreamO1ImageQuality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Standard => write!(f, "standard"),
            Self::Dev => write!(f, "dev"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

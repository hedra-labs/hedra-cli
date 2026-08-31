pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputGrokImagineResolution {
    OneK,
    TwoK,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputGrokImagineResolution {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::OneK => serializer.serialize_str("1k"),
            Self::TwoK => serializer.serialize_str("2k"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputGrokImagineResolution {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "1k" => Ok(Self::OneK),
            "2k" => Ok(Self::TwoK),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputGrokImagineResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OneK => write!(f, "1k"),
            Self::TwoK => write!(f, "2k"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

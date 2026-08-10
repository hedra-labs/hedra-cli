pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output aspect ratio.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputLtx23AspectRatio {
    Auto,
    Sixteen9,
    Nine16,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputLtx23AspectRatio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::Sixteen9 => serializer.serialize_str("16:9"),
            Self::Nine16 => serializer.serialize_str("9:16"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputLtx23AspectRatio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "auto" => Ok(Self::Auto),
            "16:9" => Ok(Self::Sixteen9),
            "9:16" => Ok(Self::Nine16),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputLtx23AspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::Sixteen9 => write!(f, "16:9"),
            Self::Nine16 => write!(f, "9:16"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

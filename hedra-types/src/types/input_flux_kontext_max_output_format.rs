pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output image format.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputFluxKontextMaxOutputFormat {
    Jpeg,
    Png,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputFluxKontextMaxOutputFormat {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Jpeg => serializer.serialize_str("jpeg"),
            Self::Png => serializer.serialize_str("png"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputFluxKontextMaxOutputFormat {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "jpeg" => Ok(Self::Jpeg),
            "png" => Ok(Self::Png),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputFluxKontextMaxOutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Jpeg => write!(f, "jpeg"),
            Self::Png => write!(f, "png"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

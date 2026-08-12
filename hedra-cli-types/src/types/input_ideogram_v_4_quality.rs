pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputIdeogramV4Quality {
    Turbo,
    Balanced,
    Quality,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputIdeogramV4Quality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Turbo => serializer.serialize_str("turbo"),
            Self::Balanced => serializer.serialize_str("balanced"),
            Self::Quality => serializer.serialize_str("quality"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputIdeogramV4Quality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "turbo" => Ok(Self::Turbo),
            "balanced" => Ok(Self::Balanced),
            "quality" => Ok(Self::Quality),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputIdeogramV4Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Turbo => write!(f, "turbo"),
            Self::Balanced => write!(f, "balanced"),
            Self::Quality => write!(f, "quality"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

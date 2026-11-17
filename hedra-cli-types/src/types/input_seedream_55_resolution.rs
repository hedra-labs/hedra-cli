pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputSeedream55Resolution {
    One5K,
    TwoK,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputSeedream55Resolution {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::One5K => serializer.serialize_str("1.5K"),
            Self::TwoK => serializer.serialize_str("2K"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputSeedream55Resolution {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "1.5K" => Ok(Self::One5K),
            "2K" => Ok(Self::TwoK),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputSeedream55Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::One5K => write!(f, "1.5K"),
            Self::TwoK => write!(f, "2K"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

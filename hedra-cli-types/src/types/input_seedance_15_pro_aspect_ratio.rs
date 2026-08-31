pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output aspect ratio.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputSeedance15ProAspectRatio {
    One1,
    Three4,
    Four3,
    Sixteen9,
    TwentyOne9,
    Nine16,
    Adaptive,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputSeedance15ProAspectRatio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::One1 => serializer.serialize_str("1:1"),
            Self::Three4 => serializer.serialize_str("3:4"),
            Self::Four3 => serializer.serialize_str("4:3"),
            Self::Sixteen9 => serializer.serialize_str("16:9"),
            Self::TwentyOne9 => serializer.serialize_str("21:9"),
            Self::Nine16 => serializer.serialize_str("9:16"),
            Self::Adaptive => serializer.serialize_str("adaptive"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputSeedance15ProAspectRatio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "1:1" => Ok(Self::One1),
            "3:4" => Ok(Self::Three4),
            "4:3" => Ok(Self::Four3),
            "16:9" => Ok(Self::Sixteen9),
            "21:9" => Ok(Self::TwentyOne9),
            "9:16" => Ok(Self::Nine16),
            "adaptive" => Ok(Self::Adaptive),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputSeedance15ProAspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::One1 => write!(f, "1:1"),
            Self::Three4 => write!(f, "3:4"),
            Self::Four3 => write!(f, "4:3"),
            Self::Sixteen9 => write!(f, "16:9"),
            Self::TwentyOne9 => write!(f, "21:9"),
            Self::Nine16 => write!(f, "9:16"),
            Self::Adaptive => write!(f, "adaptive"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

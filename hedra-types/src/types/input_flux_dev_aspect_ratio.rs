pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output aspect ratio.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputFluxDevAspectRatio {
    One1,
    One3,
    Two3,
    Three1,
    Three2,
    Three4,
    Four3,
    Sixteen9,
    Nine16,
    Ten16,
    Sixteen10,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputFluxDevAspectRatio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::One1 => serializer.serialize_str("1:1"),
            Self::One3 => serializer.serialize_str("1:3"),
            Self::Two3 => serializer.serialize_str("2:3"),
            Self::Three1 => serializer.serialize_str("3:1"),
            Self::Three2 => serializer.serialize_str("3:2"),
            Self::Three4 => serializer.serialize_str("3:4"),
            Self::Four3 => serializer.serialize_str("4:3"),
            Self::Sixteen9 => serializer.serialize_str("16:9"),
            Self::Nine16 => serializer.serialize_str("9:16"),
            Self::Ten16 => serializer.serialize_str("10:16"),
            Self::Sixteen10 => serializer.serialize_str("16:10"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputFluxDevAspectRatio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "1:1" => Ok(Self::One1),
            "1:3" => Ok(Self::One3),
            "2:3" => Ok(Self::Two3),
            "3:1" => Ok(Self::Three1),
            "3:2" => Ok(Self::Three2),
            "3:4" => Ok(Self::Three4),
            "4:3" => Ok(Self::Four3),
            "16:9" => Ok(Self::Sixteen9),
            "9:16" => Ok(Self::Nine16),
            "10:16" => Ok(Self::Ten16),
            "16:10" => Ok(Self::Sixteen10),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputFluxDevAspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::One1 => write!(f, "1:1"),
            Self::One3 => write!(f, "1:3"),
            Self::Two3 => write!(f, "2:3"),
            Self::Three1 => write!(f, "3:1"),
            Self::Three2 => write!(f, "3:2"),
            Self::Three4 => write!(f, "3:4"),
            Self::Four3 => write!(f, "4:3"),
            Self::Sixteen9 => write!(f, "16:9"),
            Self::Nine16 => write!(f, "9:16"),
            Self::Ten16 => write!(f, "10:16"),
            Self::Sixteen10 => write!(f, "16:10"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

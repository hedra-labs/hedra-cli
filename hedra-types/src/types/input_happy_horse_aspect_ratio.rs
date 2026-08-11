pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output aspect ratio.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputHappyHorseAspectRatio {
    Sixteen9,
    Nine16,
    Four3,
    Three4,
    One1,
    TwentyOne9,
    Sixteen10,
    Three2,
    Two3,
    Ten16,
    Nine21,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputHappyHorseAspectRatio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Sixteen9 => serializer.serialize_str("16:9"),
            Self::Nine16 => serializer.serialize_str("9:16"),
            Self::Four3 => serializer.serialize_str("4:3"),
            Self::Three4 => serializer.serialize_str("3:4"),
            Self::One1 => serializer.serialize_str("1:1"),
            Self::TwentyOne9 => serializer.serialize_str("21:9"),
            Self::Sixteen10 => serializer.serialize_str("16:10"),
            Self::Three2 => serializer.serialize_str("3:2"),
            Self::Two3 => serializer.serialize_str("2:3"),
            Self::Ten16 => serializer.serialize_str("10:16"),
            Self::Nine21 => serializer.serialize_str("9:21"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputHappyHorseAspectRatio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "16:9" => Ok(Self::Sixteen9),
            "9:16" => Ok(Self::Nine16),
            "4:3" => Ok(Self::Four3),
            "3:4" => Ok(Self::Three4),
            "1:1" => Ok(Self::One1),
            "21:9" => Ok(Self::TwentyOne9),
            "16:10" => Ok(Self::Sixteen10),
            "3:2" => Ok(Self::Three2),
            "2:3" => Ok(Self::Two3),
            "10:16" => Ok(Self::Ten16),
            "9:21" => Ok(Self::Nine21),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputHappyHorseAspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sixteen9 => write!(f, "16:9"),
            Self::Nine16 => write!(f, "9:16"),
            Self::Four3 => write!(f, "4:3"),
            Self::Three4 => write!(f, "3:4"),
            Self::One1 => write!(f, "1:1"),
            Self::TwentyOne9 => write!(f, "21:9"),
            Self::Sixteen10 => write!(f, "16:10"),
            Self::Three2 => write!(f, "3:2"),
            Self::Two3 => write!(f, "2:3"),
            Self::Ten16 => write!(f, "10:16"),
            Self::Nine21 => write!(f, "9:21"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

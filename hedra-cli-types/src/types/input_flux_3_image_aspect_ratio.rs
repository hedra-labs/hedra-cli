pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output aspect ratio.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputFlux3ImageAspectRatio {
    Auto,
    TwentyOne9,
    Two1,
    Sixteen9,
    Three2,
    Seven5,
    Four3,
    Five4,
    One1,
    Four5,
    Three4,
    Five7,
    Two3,
    Nine16,
    One2,
    Nine21,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputFlux3ImageAspectRatio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::TwentyOne9 => serializer.serialize_str("21:9"),
            Self::Two1 => serializer.serialize_str("2:1"),
            Self::Sixteen9 => serializer.serialize_str("16:9"),
            Self::Three2 => serializer.serialize_str("3:2"),
            Self::Seven5 => serializer.serialize_str("7:5"),
            Self::Four3 => serializer.serialize_str("4:3"),
            Self::Five4 => serializer.serialize_str("5:4"),
            Self::One1 => serializer.serialize_str("1:1"),
            Self::Four5 => serializer.serialize_str("4:5"),
            Self::Three4 => serializer.serialize_str("3:4"),
            Self::Five7 => serializer.serialize_str("5:7"),
            Self::Two3 => serializer.serialize_str("2:3"),
            Self::Nine16 => serializer.serialize_str("9:16"),
            Self::One2 => serializer.serialize_str("1:2"),
            Self::Nine21 => serializer.serialize_str("9:21"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputFlux3ImageAspectRatio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "auto" => Ok(Self::Auto),
            "21:9" => Ok(Self::TwentyOne9),
            "2:1" => Ok(Self::Two1),
            "16:9" => Ok(Self::Sixteen9),
            "3:2" => Ok(Self::Three2),
            "7:5" => Ok(Self::Seven5),
            "4:3" => Ok(Self::Four3),
            "5:4" => Ok(Self::Five4),
            "1:1" => Ok(Self::One1),
            "4:5" => Ok(Self::Four5),
            "3:4" => Ok(Self::Three4),
            "5:7" => Ok(Self::Five7),
            "2:3" => Ok(Self::Two3),
            "9:16" => Ok(Self::Nine16),
            "1:2" => Ok(Self::One2),
            "9:21" => Ok(Self::Nine21),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputFlux3ImageAspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auto => write!(f, "auto"),
            Self::TwentyOne9 => write!(f, "21:9"),
            Self::Two1 => write!(f, "2:1"),
            Self::Sixteen9 => write!(f, "16:9"),
            Self::Three2 => write!(f, "3:2"),
            Self::Seven5 => write!(f, "7:5"),
            Self::Four3 => write!(f, "4:3"),
            Self::Five4 => write!(f, "5:4"),
            Self::One1 => write!(f, "1:1"),
            Self::Four5 => write!(f, "4:5"),
            Self::Three4 => write!(f, "3:4"),
            Self::Five7 => write!(f, "5:7"),
            Self::Two3 => write!(f, "2:3"),
            Self::Nine16 => write!(f, "9:16"),
            Self::One2 => write!(f, "1:2"),
            Self::Nine21 => write!(f, "9:21"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

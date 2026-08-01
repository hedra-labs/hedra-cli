pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output aspect ratio.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputReve21EditAspectRatio {
    Four1,
    Three1,
    TwentyOne9,
    Two1,
    Seventeen9,
    Sixteen9,
    Three2,
    Four3,
    Five4,
    One1,
    Four5,
    Three4,
    Two3,
    Nine16,
    One2,
    One3,
    One4,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputReve21EditAspectRatio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Four1 => serializer.serialize_str("4:1"),
            Self::Three1 => serializer.serialize_str("3:1"),
            Self::TwentyOne9 => serializer.serialize_str("21:9"),
            Self::Two1 => serializer.serialize_str("2:1"),
            Self::Seventeen9 => serializer.serialize_str("17:9"),
            Self::Sixteen9 => serializer.serialize_str("16:9"),
            Self::Three2 => serializer.serialize_str("3:2"),
            Self::Four3 => serializer.serialize_str("4:3"),
            Self::Five4 => serializer.serialize_str("5:4"),
            Self::One1 => serializer.serialize_str("1:1"),
            Self::Four5 => serializer.serialize_str("4:5"),
            Self::Three4 => serializer.serialize_str("3:4"),
            Self::Two3 => serializer.serialize_str("2:3"),
            Self::Nine16 => serializer.serialize_str("9:16"),
            Self::One2 => serializer.serialize_str("1:2"),
            Self::One3 => serializer.serialize_str("1:3"),
            Self::One4 => serializer.serialize_str("1:4"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputReve21EditAspectRatio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "4:1" => Ok(Self::Four1),
            "3:1" => Ok(Self::Three1),
            "21:9" => Ok(Self::TwentyOne9),
            "2:1" => Ok(Self::Two1),
            "17:9" => Ok(Self::Seventeen9),
            "16:9" => Ok(Self::Sixteen9),
            "3:2" => Ok(Self::Three2),
            "4:3" => Ok(Self::Four3),
            "5:4" => Ok(Self::Five4),
            "1:1" => Ok(Self::One1),
            "4:5" => Ok(Self::Four5),
            "3:4" => Ok(Self::Three4),
            "2:3" => Ok(Self::Two3),
            "9:16" => Ok(Self::Nine16),
            "1:2" => Ok(Self::One2),
            "1:3" => Ok(Self::One3),
            "1:4" => Ok(Self::One4),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputReve21EditAspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Four1 => write!(f, "4:1"),
            Self::Three1 => write!(f, "3:1"),
            Self::TwentyOne9 => write!(f, "21:9"),
            Self::Two1 => write!(f, "2:1"),
            Self::Seventeen9 => write!(f, "17:9"),
            Self::Sixteen9 => write!(f, "16:9"),
            Self::Three2 => write!(f, "3:2"),
            Self::Four3 => write!(f, "4:3"),
            Self::Five4 => write!(f, "5:4"),
            Self::One1 => write!(f, "1:1"),
            Self::Four5 => write!(f, "4:5"),
            Self::Three4 => write!(f, "3:4"),
            Self::Two3 => write!(f, "2:3"),
            Self::Nine16 => write!(f, "9:16"),
            Self::One2 => write!(f, "1:2"),
            Self::One3 => write!(f, "1:3"),
            Self::One4 => write!(f, "1:4"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

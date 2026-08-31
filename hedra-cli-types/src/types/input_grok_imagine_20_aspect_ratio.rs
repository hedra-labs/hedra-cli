pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output aspect ratio.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputGrokImagine20AspectRatio {
    Two1,
    Twenty9,
    Nineteen59,
    Sixteen9,
    Four3,
    Three2,
    One1,
    Two3,
    Three4,
    Nine16,
    Nine195,
    Nine20,
    One2,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputGrokImagine20AspectRatio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Two1 => serializer.serialize_str("2:1"),
            Self::Twenty9 => serializer.serialize_str("20:9"),
            Self::Nineteen59 => serializer.serialize_str("19.5:9"),
            Self::Sixteen9 => serializer.serialize_str("16:9"),
            Self::Four3 => serializer.serialize_str("4:3"),
            Self::Three2 => serializer.serialize_str("3:2"),
            Self::One1 => serializer.serialize_str("1:1"),
            Self::Two3 => serializer.serialize_str("2:3"),
            Self::Three4 => serializer.serialize_str("3:4"),
            Self::Nine16 => serializer.serialize_str("9:16"),
            Self::Nine195 => serializer.serialize_str("9:19.5"),
            Self::Nine20 => serializer.serialize_str("9:20"),
            Self::One2 => serializer.serialize_str("1:2"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputGrokImagine20AspectRatio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "2:1" => Ok(Self::Two1),
            "20:9" => Ok(Self::Twenty9),
            "19.5:9" => Ok(Self::Nineteen59),
            "16:9" => Ok(Self::Sixteen9),
            "4:3" => Ok(Self::Four3),
            "3:2" => Ok(Self::Three2),
            "1:1" => Ok(Self::One1),
            "2:3" => Ok(Self::Two3),
            "3:4" => Ok(Self::Three4),
            "9:16" => Ok(Self::Nine16),
            "9:19.5" => Ok(Self::Nine195),
            "9:20" => Ok(Self::Nine20),
            "1:2" => Ok(Self::One2),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputGrokImagine20AspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Two1 => write!(f, "2:1"),
            Self::Twenty9 => write!(f, "20:9"),
            Self::Nineteen59 => write!(f, "19.5:9"),
            Self::Sixteen9 => write!(f, "16:9"),
            Self::Four3 => write!(f, "4:3"),
            Self::Three2 => write!(f, "3:2"),
            Self::One1 => write!(f, "1:1"),
            Self::Two3 => write!(f, "2:3"),
            Self::Three4 => write!(f, "3:4"),
            Self::Nine16 => write!(f, "9:16"),
            Self::Nine195 => write!(f, "9:19.5"),
            Self::Nine20 => write!(f, "9:20"),
            Self::One2 => write!(f, "1:2"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

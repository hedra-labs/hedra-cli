pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output aspect ratio. 'adaptive' lets the model size the output itself — matching the source image when you pass one.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputNanoBananaProAspectRatio {
    Adaptive,
    Sixteen9,
    Nine16,
    One1,
    TwentyOne9,
    Four3,
    Three4,
    Three2,
    Two3,
    Five4,
    Four5,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputNanoBananaProAspectRatio {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Adaptive => serializer.serialize_str("adaptive"),
            Self::Sixteen9 => serializer.serialize_str("16:9"),
            Self::Nine16 => serializer.serialize_str("9:16"),
            Self::One1 => serializer.serialize_str("1:1"),
            Self::TwentyOne9 => serializer.serialize_str("21:9"),
            Self::Four3 => serializer.serialize_str("4:3"),
            Self::Three4 => serializer.serialize_str("3:4"),
            Self::Three2 => serializer.serialize_str("3:2"),
            Self::Two3 => serializer.serialize_str("2:3"),
            Self::Five4 => serializer.serialize_str("5:4"),
            Self::Four5 => serializer.serialize_str("4:5"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputNanoBananaProAspectRatio {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "adaptive" => Ok(Self::Adaptive),
            "16:9" => Ok(Self::Sixteen9),
            "9:16" => Ok(Self::Nine16),
            "1:1" => Ok(Self::One1),
            "21:9" => Ok(Self::TwentyOne9),
            "4:3" => Ok(Self::Four3),
            "3:4" => Ok(Self::Three4),
            "3:2" => Ok(Self::Three2),
            "2:3" => Ok(Self::Two3),
            "5:4" => Ok(Self::Five4),
            "4:5" => Ok(Self::Four5),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputNanoBananaProAspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Adaptive => write!(f, "adaptive"),
            Self::Sixteen9 => write!(f, "16:9"),
            Self::Nine16 => write!(f, "9:16"),
            Self::One1 => write!(f, "1:1"),
            Self::TwentyOne9 => write!(f, "21:9"),
            Self::Four3 => write!(f, "4:3"),
            Self::Three4 => write!(f, "3:4"),
            Self::Three2 => write!(f, "3:2"),
            Self::Two3 => write!(f, "2:3"),
            Self::Five4 => write!(f, "5:4"),
            Self::Four5 => write!(f, "4:5"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

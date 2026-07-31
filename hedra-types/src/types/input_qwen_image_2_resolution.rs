pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputQwenImage2Resolution {
    FiveHundredFortyP,
    SevenHundredTwentyP,
    OneThousandEightyP,
    OneThousandFourHundredFortyP2KQhd,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputQwenImage2Resolution {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::FiveHundredFortyP => serializer.serialize_str("540p"),
            Self::SevenHundredTwentyP => serializer.serialize_str("720p"),
            Self::OneThousandEightyP => serializer.serialize_str("1080p"),
            Self::OneThousandFourHundredFortyP2KQhd => serializer.serialize_str("1440p (2K QHD)"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputQwenImage2Resolution {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "540p" => Ok(Self::FiveHundredFortyP),
            "720p" => Ok(Self::SevenHundredTwentyP),
            "1080p" => Ok(Self::OneThousandEightyP),
            "1440p (2K QHD)" => Ok(Self::OneThousandFourHundredFortyP2KQhd),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputQwenImage2Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FiveHundredFortyP => write!(f, "540p"),
            Self::SevenHundredTwentyP => write!(f, "720p"),
            Self::OneThousandEightyP => write!(f, "1080p"),
            Self::OneThousandFourHundredFortyP2KQhd => write!(f, "1440p (2K QHD)"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

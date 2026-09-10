pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputGeminiOmniFlash11Resolution {
    ThreeHundredSixtyP,
    SevenHundredTwentyP,
    OneThousandEightyP,
    FourK,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputGeminiOmniFlash11Resolution {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::ThreeHundredSixtyP => serializer.serialize_str("360p"),
            Self::SevenHundredTwentyP => serializer.serialize_str("720p"),
            Self::OneThousandEightyP => serializer.serialize_str("1080p"),
            Self::FourK => serializer.serialize_str("4k"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputGeminiOmniFlash11Resolution {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "360p" => Ok(Self::ThreeHundredSixtyP),
            "720p" => Ok(Self::SevenHundredTwentyP),
            "1080p" => Ok(Self::OneThousandEightyP),
            "4k" => Ok(Self::FourK),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputGeminiOmniFlash11Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ThreeHundredSixtyP => write!(f, "360p"),
            Self::SevenHundredTwentyP => write!(f, "720p"),
            Self::OneThousandEightyP => write!(f, "1080p"),
            Self::FourK => write!(f, "4k"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

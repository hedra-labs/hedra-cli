pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputViduQ3Resolution {
    FiveHundredFortyP,
    SevenHundredTwentyP,
    OneThousandEightyP,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputViduQ3Resolution {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::FiveHundredFortyP => serializer.serialize_str("540p"),
            Self::SevenHundredTwentyP => serializer.serialize_str("720p"),
            Self::OneThousandEightyP => serializer.serialize_str("1080p"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputViduQ3Resolution {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "540p" => Ok(Self::FiveHundredFortyP),
            "720p" => Ok(Self::SevenHundredTwentyP),
            "1080p" => Ok(Self::OneThousandEightyP),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputViduQ3Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FiveHundredFortyP => write!(f, "540p"),
            Self::SevenHundredTwentyP => write!(f, "720p"),
            Self::OneThousandEightyP => write!(f, "1080p"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

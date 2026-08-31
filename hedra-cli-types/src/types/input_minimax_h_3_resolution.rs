pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputMinimaxH3Resolution {
    FourHundredEightyP,
    SevenHundredSixtyEightP,
    TwoK,
    FourK,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputMinimaxH3Resolution {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::FourHundredEightyP => serializer.serialize_str("480p"),
            Self::SevenHundredSixtyEightP => serializer.serialize_str("768p"),
            Self::TwoK => serializer.serialize_str("2K"),
            Self::FourK => serializer.serialize_str("4K"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputMinimaxH3Resolution {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "480p" => Ok(Self::FourHundredEightyP),
            "768p" => Ok(Self::SevenHundredSixtyEightP),
            "2K" => Ok(Self::TwoK),
            "4K" => Ok(Self::FourK),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputMinimaxH3Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FourHundredEightyP => write!(f, "480p"),
            Self::SevenHundredSixtyEightP => write!(f, "768p"),
            Self::TwoK => write!(f, "2K"),
            Self::FourK => write!(f, "4K"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

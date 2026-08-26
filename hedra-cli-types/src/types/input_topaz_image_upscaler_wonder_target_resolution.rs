pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Short edge of the upscaled image. The long edge follows the source's aspect ratio.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputTopazImageUpscalerWonderTargetResolution {
    OneThousandEightyP,
    TwoK,
    FourK,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputTopazImageUpscalerWonderTargetResolution {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::OneThousandEightyP => serializer.serialize_str("1080p"),
            Self::TwoK => serializer.serialize_str("2K"),
            Self::FourK => serializer.serialize_str("4K"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputTopazImageUpscalerWonderTargetResolution {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "1080p" => Ok(Self::OneThousandEightyP),
            "2K" => Ok(Self::TwoK),
            "4K" => Ok(Self::FourK),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputTopazImageUpscalerWonderTargetResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OneThousandEightyP => write!(f, "1080p"),
            Self::TwoK => write!(f, "2K"),
            Self::FourK => write!(f, "4K"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// How a multi-shot storyboard is cut. 'customize' honours each shot's declared duration; 'intelligent' lets the model determine the shot structure. Ignored unless `multi_prompt` is supplied.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputKlingO3ShotType {
    Customize,
    Intelligent,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputKlingO3ShotType {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Customize => serializer.serialize_str("customize"),
            Self::Intelligent => serializer.serialize_str("intelligent"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputKlingO3ShotType {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "customize" => Ok(Self::Customize),
            "intelligent" => Ok(Self::Intelligent),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputKlingO3ShotType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Customize => write!(f, "customize"),
            Self::Intelligent => write!(f, "intelligent"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

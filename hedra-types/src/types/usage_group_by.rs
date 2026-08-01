pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum UsageGroupBy {
    Total,
    Day,
    Model,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for UsageGroupBy {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Total => serializer.serialize_str("total"),
            Self::Day => serializer.serialize_str("day"),
            Self::Model => serializer.serialize_str("model"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for UsageGroupBy {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "total" => Ok(Self::Total),
            "day" => Ok(Self::Day),
            "model" => Ok(Self::Model),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for UsageGroupBy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Total => write!(f, "total"),
            Self::Day => write!(f, "day"),
            Self::Model => write!(f, "model"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

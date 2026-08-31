pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The `gender` curation label, as a voice search filter.
/// 
/// Two members because the label carries exactly two values across every
/// provider; published as an enum so a generated SDK types the filter rather
/// than taking any string and failing at the boundary.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VoiceGender {
    Male,
    Female,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for VoiceGender {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Male => serializer.serialize_str("male"),
            Self::Female => serializer.serialize_str("female"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for VoiceGender {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "male" => Ok(Self::Male),
            "female" => Ok(Self::Female),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for VoiceGender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Male => write!(f, "male"),
            Self::Female => write!(f, "female"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

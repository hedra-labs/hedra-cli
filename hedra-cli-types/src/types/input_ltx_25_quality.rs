pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Quality level to generate at. `fast` — tuned for turnaround, and the only level that renders 1440p, 2160p, or clips past 10 seconds. `pro` — the higher-fidelity tier, for final output at 720p or 1080p, and the level that caps both the clip it renders and the audio driving it at 10 seconds.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputLtx25Quality {
    Fast,
    Pro,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for InputLtx25Quality {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Fast => serializer.serialize_str("fast"),
            Self::Pro => serializer.serialize_str("pro"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for InputLtx25Quality {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "fast" => Ok(Self::Fast),
            "pro" => Ok(Self::Pro),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for InputLtx25Quality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fast => write!(f, "fast"),
            Self::Pro => write!(f, "pro"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// The instance failures v3 publishes; `platform_error` means the platform lost
/// the instance or could not stage it, so creating the sandbox again may succeed,
/// and `other` covers reasons hedra-compute adds later.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SandboxFailureReason {
    QuotaExceeded,
    NoCapacity,
    ImagePullFailed,
    StartFailed,
    ProcessExited,
    OomKilled,
    GpuError,
    PlatformError,
    Other,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for SandboxFailureReason {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::QuotaExceeded => serializer.serialize_str("quota_exceeded"),
            Self::NoCapacity => serializer.serialize_str("no_capacity"),
            Self::ImagePullFailed => serializer.serialize_str("image_pull_failed"),
            Self::StartFailed => serializer.serialize_str("start_failed"),
            Self::ProcessExited => serializer.serialize_str("process_exited"),
            Self::OomKilled => serializer.serialize_str("oom_killed"),
            Self::GpuError => serializer.serialize_str("gpu_error"),
            Self::PlatformError => serializer.serialize_str("platform_error"),
            Self::Other => serializer.serialize_str("other"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for SandboxFailureReason {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "quota_exceeded" => Ok(Self::QuotaExceeded),
            "no_capacity" => Ok(Self::NoCapacity),
            "image_pull_failed" => Ok(Self::ImagePullFailed),
            "start_failed" => Ok(Self::StartFailed),
            "process_exited" => Ok(Self::ProcessExited),
            "oom_killed" => Ok(Self::OomKilled),
            "gpu_error" => Ok(Self::GpuError),
            "platform_error" => Ok(Self::PlatformError),
            "other" => Ok(Self::Other),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for SandboxFailureReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::QuotaExceeded => write!(f, "quota_exceeded"),
            Self::NoCapacity => write!(f, "no_capacity"),
            Self::ImagePullFailed => write!(f, "image_pull_failed"),
            Self::StartFailed => write!(f, "start_failed"),
            Self::ProcessExited => write!(f, "process_exited"),
            Self::OomKilled => write!(f, "oom_killed"),
            Self::GpuError => write!(f, "gpu_error"),
            Self::PlatformError => write!(f, "platform_error"),
            Self::Other => write!(f, "other"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum JobLogEvent {
    Queued,
    Started,
    ModerationPassed,
    ProviderSubmitted,
    ProviderError,
    RetryScheduled,
    Progress,
    Finalizing,
    DownloadReady,
    Completed,
    Failed,
    Recovered,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for JobLogEvent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Queued => serializer.serialize_str("queued"),
            Self::Started => serializer.serialize_str("started"),
            Self::ModerationPassed => serializer.serialize_str("moderation.passed"),
            Self::ProviderSubmitted => serializer.serialize_str("provider.submitted"),
            Self::ProviderError => serializer.serialize_str("provider.error"),
            Self::RetryScheduled => serializer.serialize_str("retry.scheduled"),
            Self::Progress => serializer.serialize_str("progress"),
            Self::Finalizing => serializer.serialize_str("finalizing"),
            Self::DownloadReady => serializer.serialize_str("download.ready"),
            Self::Completed => serializer.serialize_str("completed"),
            Self::Failed => serializer.serialize_str("failed"),
            Self::Recovered => serializer.serialize_str("recovered"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for JobLogEvent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "queued" => Ok(Self::Queued),
            "started" => Ok(Self::Started),
            "moderation.passed" => Ok(Self::ModerationPassed),
            "provider.submitted" => Ok(Self::ProviderSubmitted),
            "provider.error" => Ok(Self::ProviderError),
            "retry.scheduled" => Ok(Self::RetryScheduled),
            "progress" => Ok(Self::Progress),
            "finalizing" => Ok(Self::Finalizing),
            "download.ready" => Ok(Self::DownloadReady),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "recovered" => Ok(Self::Recovered),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for JobLogEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Started => write!(f, "started"),
            Self::ModerationPassed => write!(f, "moderation.passed"),
            Self::ProviderSubmitted => write!(f, "provider.submitted"),
            Self::ProviderError => write!(f, "provider.error"),
            Self::RetryScheduled => write!(f, "retry.scheduled"),
            Self::Progress => write!(f, "progress"),
            Self::Finalizing => write!(f, "finalizing"),
            Self::DownloadReady => write!(f, "download.ready"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Recovered => write!(f, "recovered"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

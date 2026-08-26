pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Least-privilege grants carried by v3 API keys.
/// 
/// Stored as text[] on api_keys; NULL scopes means full access (legacy keys).
/// Scopes are stored as text so new least-privilege surfaces can be introduced
/// without a database-enum migration.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApiKeyScope {
    JobsRead,
    JobsWrite,
    ModelsRead,
    FilesWrite,
    WebhooksManage,
    LogDrainsManage,
    UsageRead,
    KeysManage,
    ChatWrite,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for ApiKeyScope {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::JobsRead => serializer.serialize_str("jobs:read"),
            Self::JobsWrite => serializer.serialize_str("jobs:write"),
            Self::ModelsRead => serializer.serialize_str("models:read"),
            Self::FilesWrite => serializer.serialize_str("files:write"),
            Self::WebhooksManage => serializer.serialize_str("webhooks:manage"),
            Self::LogDrainsManage => serializer.serialize_str("log_drains:manage"),
            Self::UsageRead => serializer.serialize_str("usage:read"),
            Self::KeysManage => serializer.serialize_str("keys:manage"),
            Self::ChatWrite => serializer.serialize_str("chat:write"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for ApiKeyScope {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "jobs:read" => Ok(Self::JobsRead),
            "jobs:write" => Ok(Self::JobsWrite),
            "models:read" => Ok(Self::ModelsRead),
            "files:write" => Ok(Self::FilesWrite),
            "webhooks:manage" => Ok(Self::WebhooksManage),
            "log_drains:manage" => Ok(Self::LogDrainsManage),
            "usage:read" => Ok(Self::UsageRead),
            "keys:manage" => Ok(Self::KeysManage),
            "chat:write" => Ok(Self::ChatWrite),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for ApiKeyScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JobsRead => write!(f, "jobs:read"),
            Self::JobsWrite => write!(f, "jobs:write"),
            Self::ModelsRead => write!(f, "models:read"),
            Self::FilesWrite => write!(f, "files:write"),
            Self::WebhooksManage => write!(f, "webhooks:manage"),
            Self::LogDrainsManage => write!(f, "log_drains:manage"),
            Self::UsageRead => write!(f, "usage:read"),
            Self::KeysManage => write!(f, "keys:manage"),
            Self::ChatWrite => write!(f, "chat:write"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

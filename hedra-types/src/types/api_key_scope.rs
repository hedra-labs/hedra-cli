pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Least-privilege grants carried by v3 API keys.
/// 
/// Stored as text[] on api_keys; NULL scopes means full access (legacy keys).
/// Scopes without a live v3 surface yet (webhooks/usage) are accepted at
/// key creation for forward compatibility and enforced when those surfaces ship.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApiKeyScope {
    GenerationsWrite,
    RequestsRead,
    ModelsRead,
    FilesWrite,
    WebhooksManage,
    UsageRead,
    KeysManage,
    /// This variant is used for forward compatibility.
    /// If the server sends a value not recognized by the current SDK version,
    /// it will be captured here with the raw string value.
    __Unknown(String),
}
impl Serialize for ApiKeyScope {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::GenerationsWrite => serializer.serialize_str("generations:write"),
            Self::RequestsRead => serializer.serialize_str("requests:read"),
            Self::ModelsRead => serializer.serialize_str("models:read"),
            Self::FilesWrite => serializer.serialize_str("files:write"),
            Self::WebhooksManage => serializer.serialize_str("webhooks:manage"),
            Self::UsageRead => serializer.serialize_str("usage:read"),
            Self::KeysManage => serializer.serialize_str("keys:manage"),
            Self::__Unknown(val) => serializer.serialize_str(val),
        }
    }
}

impl<'de> Deserialize<'de> for ApiKeyScope {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        match value.as_str() {
            "generations:write" => Ok(Self::GenerationsWrite),
            "requests:read" => Ok(Self::RequestsRead),
            "models:read" => Ok(Self::ModelsRead),
            "files:write" => Ok(Self::FilesWrite),
            "webhooks:manage" => Ok(Self::WebhooksManage),
            "usage:read" => Ok(Self::UsageRead),
            "keys:manage" => Ok(Self::KeysManage),
            _ => Ok(Self::__Unknown(value)),
        }
    }
}

impl fmt::Display for ApiKeyScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GenerationsWrite => write!(f, "generations:write"),
            Self::RequestsRead => write!(f, "requests:read"),
            Self::ModelsRead => write!(f, "models:read"),
            Self::FilesWrite => write!(f, "files:write"),
            Self::WebhooksManage => write!(f, "webhooks:manage"),
            Self::UsageRead => write!(f, "usage:read"),
            Self::KeysManage => write!(f, "keys:manage"),
            Self::__Unknown(val) => write!(f, "{}", val),
        }
    }
}

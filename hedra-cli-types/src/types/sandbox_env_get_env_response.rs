pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum GetEnvResponse {
        #[serde(rename = "config")]
        #[non_exhaustive]
        Config {
            #[serde(default)]
            name: String,
            #[serde(default)]
            #[serde(with = "crate::core::flexible_datetime::offset")]
            created_at: DateTime<FixedOffset>,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(default)]
            #[serde(with = "crate::core::flexible_datetime::offset::option")]
            updated_at: Option<DateTime<FixedOffset>>,
            #[serde(default)]
            value: String,
        },

        #[serde(rename = "secret")]
        #[non_exhaustive]
        Secret {
            #[serde(default)]
            name: String,
            #[serde(default)]
            #[serde(with = "crate::core::flexible_datetime::offset")]
            created_at: DateTime<FixedOffset>,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(default)]
            #[serde(with = "crate::core::flexible_datetime::offset::option")]
            updated_at: Option<DateTime<FixedOffset>>,
        },

        /// Catch-all variant for unrecognized discriminant values.
        /// If the server sends a discriminant not recognized by the current SDK
        /// version, the raw payload is captured here so callers can still inspect it.
        #[serde(untagged)]
        __Unknown(serde_json::Value),
}

impl GetEnvResponse {
    pub fn config(name: String, created_at: DateTime<FixedOffset>, value: String) -> Self {
        Self::Config { name, created_at, updated_at: None, value }
    }

    pub fn secret(name: String, created_at: DateTime<FixedOffset>) -> Self {
        Self::Secret { name, created_at, updated_at: None }
    }

    pub fn config_with_updated_at(name: String, created_at: DateTime<FixedOffset>, updated_at: DateTime<FixedOffset>, value: String) -> Self {
        Self::Config { name, created_at, updated_at: Some(updated_at), value }
    }

    pub fn secret_with_updated_at(name: String, created_at: DateTime<FixedOffset>, updated_at: DateTime<FixedOffset>) -> Self {
        Self::Secret { name, created_at, updated_at: Some(updated_at) }
    }

    pub fn unknown(value: serde_json::Value) -> Self {
        Self::__Unknown(value)
    }
}

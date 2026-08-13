pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "source")]
#[non_exhaustive]
pub enum InputNanoBananaImagesItem {
        #[serde(rename = "url")]
        #[non_exhaustive]
        Url {
            #[serde(default)]
            url: String,
        },

        #[serde(rename = "asset")]
        #[non_exhaustive]
        Asset {
            #[serde(default)]
            asset_id: String,
        },

        /// Catch-all variant for unrecognized discriminant values.
        /// If the server sends a discriminant not recognized by the current SDK
        /// version, the raw payload is captured here so callers can still inspect it.
        #[serde(untagged)]
        __Unknown(serde_json::Value),
}

impl InputNanoBananaImagesItem {
    pub fn url(url: String) -> Self {
        Self::Url { url }
    }

    pub fn asset(asset_id: String) -> Self {
        Self::Asset { asset_id }
    }

    pub fn unknown(value: serde_json::Value) -> Self {
        Self::__Unknown(value)
    }
}

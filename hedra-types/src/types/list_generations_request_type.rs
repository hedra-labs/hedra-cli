pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum ListGenerationsRequestType {
        AssetType(AssetType),

        GenerationType(GenerationType),
}

impl ListGenerationsRequestType {
    pub fn is_asset_type(&self) -> bool {
        matches!(self, Self::AssetType(_))
    }

    pub fn is_generation_type(&self) -> bool {
        matches!(self, Self::GenerationType(_))
    }


    pub fn as_asset_type(&self) -> Option<&AssetType> {
        match self {
                    Self::AssetType(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_asset_type(self) -> Option<AssetType> {
        match self {
                    Self::AssetType(value) => Some(value),
                    _ => None,
                }
    }

    pub fn as_generation_type(&self) -> Option<&GenerationType> {
        match self {
                    Self::GenerationType(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_generation_type(self) -> Option<GenerationType> {
        match self {
                    Self::GenerationType(value) => Some(value),
                    _ => None,
                }
    }
}

impl fmt::Display for ListGenerationsRequestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssetType(value) => write!(f, "{}", serde_json::to_string(value).unwrap_or_else(|_| format!("{:?}", value))),
            Self::GenerationType(value) => write!(f, "{}", serde_json::to_string(value).unwrap_or_else(|_| format!("{:?}", value))),
        }
    }
}

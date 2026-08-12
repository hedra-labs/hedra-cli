pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum InputHedraAvatarBoundingBoxTarget {
        ValueList(Vec<serde_json::Value>),

        ValueListList(Vec<Vec<serde_json::Value>>),
}

impl InputHedraAvatarBoundingBoxTarget {
    pub fn is_value_list(&self) -> bool {
        matches!(self, Self::ValueList(_))
    }

    pub fn is_value_list_list(&self) -> bool {
        matches!(self, Self::ValueListList(_))
    }


    pub fn as_value_list(&self) -> Option<&Vec<serde_json::Value>> {
        match self {
                    Self::ValueList(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_value_list(self) -> Option<Vec<serde_json::Value>> {
        match self {
                    Self::ValueList(value) => Some(value),
                    _ => None,
                }
    }

    pub fn as_value_list_list(&self) -> Option<&Vec<Vec<serde_json::Value>>> {
        match self {
                    Self::ValueListList(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_value_list_list(self) -> Option<Vec<Vec<serde_json::Value>>> {
        match self {
                    Self::ValueListList(value) => Some(value),
                    _ => None,
                }
    }
}

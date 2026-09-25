pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LlmModelObjectObject {
    #[serde(rename = "model")]
    Model,
}
impl fmt::Display for LlmModelObjectObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Model => "model",
        };
        write!(f, "{}", s)
    }
}

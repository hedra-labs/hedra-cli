pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LlmModelListObject {
    #[serde(rename = "list")]
    List,
}
impl fmt::Display for LlmModelListObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::List => "list",
        };
        write!(f, "{}", s)
    }
}

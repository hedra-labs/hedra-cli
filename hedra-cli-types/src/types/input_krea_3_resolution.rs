pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InputKrea3Resolution {
    #[serde(rename = "2K")]
    TwoK,
}
impl fmt::Display for InputKrea3Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::TwoK => "2K",
        };
        write!(f, "{}", s)
    }
}

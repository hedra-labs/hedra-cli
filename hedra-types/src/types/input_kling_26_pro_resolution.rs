pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InputKling26ProResolution {
    #[serde(rename = "1080p")]
    OneThousandEightyP,
}
impl fmt::Display for InputKling26ProResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::OneThousandEightyP => "1080p",
        };
        write!(f, "{}", s)
    }
}

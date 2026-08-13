pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InputVeo2Resolution {
    #[serde(rename = "720p")]
    SevenHundredTwentyP,
}
impl fmt::Display for InputVeo2Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::SevenHundredTwentyP => "720p",
        };
        write!(f, "{}", s)
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InputKling21MasterResolution {
    #[serde(rename = "720p")]
    SevenHundredTwentyP,
}
impl fmt::Display for InputKling21MasterResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::SevenHundredTwentyP => "720p",
        };
        write!(f, "{}", s)
    }
}

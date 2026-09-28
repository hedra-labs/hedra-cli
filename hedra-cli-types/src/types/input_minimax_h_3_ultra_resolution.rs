pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InputMinimaxH3UltraResolution {
    #[serde(rename = "768p")]
    SevenHundredSixtyEightP,
}
impl fmt::Display for InputMinimaxH3UltraResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::SevenHundredSixtyEightP => "768p",
        };
        write!(f, "{}", s)
    }
}

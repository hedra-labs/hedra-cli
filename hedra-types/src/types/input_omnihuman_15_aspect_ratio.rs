pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output aspect ratio.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InputOmnihuman15AspectRatio {
    #[serde(rename = "16:9")]
    Sixteen9,
}
impl fmt::Display for InputOmnihuman15AspectRatio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Sixteen9 => "16:9",
        };
        write!(f, "{}", s)
    }
}

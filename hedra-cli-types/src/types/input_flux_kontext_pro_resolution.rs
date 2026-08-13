pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InputFluxKontextProResolution {
    #[serde(rename = "fixed")]
    Fixed,
}
impl fmt::Display for InputFluxKontextProResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Fixed => "fixed",
        };
        write!(f, "{}", s)
    }
}

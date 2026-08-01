pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Output resolution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum InputImagen3Resolution {
    #[serde(rename = "fixed")]
    Fixed,
}
impl fmt::Display for InputImagen3Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Fixed => "fixed",
        };
        write!(f, "{}", s)
    }
}

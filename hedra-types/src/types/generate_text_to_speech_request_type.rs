pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum GenerateTextToSpeechRequestType {
    #[serde(rename = "text_to_speech")]
    TextToSpeech,
}
impl fmt::Display for GenerateTextToSpeechRequestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::TextToSpeech => "text_to_speech",
        };
        write!(f, "{}", s)
    }
}

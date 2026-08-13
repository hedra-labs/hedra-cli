pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum InputHedraAvatarAudio {
        InputHedraAvatarAudioZero(InputHedraAvatarAudioZero),

        InputHedraAvatarAudioOneItemList(Vec<InputHedraAvatarAudioOneItem>),
}

impl InputHedraAvatarAudio {
    pub fn is_input_hedra_avatar_audio_zero(&self) -> bool {
        matches!(self, Self::InputHedraAvatarAudioZero(_))
    }

    pub fn is_input_hedra_avatar_audio_one_item_list(&self) -> bool {
        matches!(self, Self::InputHedraAvatarAudioOneItemList(_))
    }


    pub fn as_input_hedra_avatar_audio_zero(&self) -> Option<&InputHedraAvatarAudioZero> {
        match self {
                    Self::InputHedraAvatarAudioZero(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_input_hedra_avatar_audio_zero(self) -> Option<InputHedraAvatarAudioZero> {
        match self {
                    Self::InputHedraAvatarAudioZero(value) => Some(value),
                    _ => None,
                }
    }

    pub fn as_input_hedra_avatar_audio_one_item_list(&self) -> Option<&Vec<InputHedraAvatarAudioOneItem>> {
        match self {
                    Self::InputHedraAvatarAudioOneItemList(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_input_hedra_avatar_audio_one_item_list(self) -> Option<Vec<InputHedraAvatarAudioOneItem>> {
        match self {
                    Self::InputHedraAvatarAudioOneItemList(value) => Some(value),
                    _ => None,
                }
    }
}

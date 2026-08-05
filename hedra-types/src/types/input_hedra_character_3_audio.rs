pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum InputHedraCharacter3Audio {
        InputHedraCharacter3AudioZero(InputHedraCharacter3AudioZero),

        InputHedraCharacter3AudioOneItemList(Vec<InputHedraCharacter3AudioOneItem>),
}

impl InputHedraCharacter3Audio {
    pub fn is_input_hedra_character3audio_zero(&self) -> bool {
        matches!(self, Self::InputHedraCharacter3AudioZero(_))
    }

    pub fn is_input_hedra_character3audio_one_item_list(&self) -> bool {
        matches!(self, Self::InputHedraCharacter3AudioOneItemList(_))
    }


    pub fn as_input_hedra_character3audio_zero(&self) -> Option<&InputHedraCharacter3AudioZero> {
        match self {
                    Self::InputHedraCharacter3AudioZero(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_input_hedra_character3audio_zero(self) -> Option<InputHedraCharacter3AudioZero> {
        match self {
                    Self::InputHedraCharacter3AudioZero(value) => Some(value),
                    _ => None,
                }
    }

    pub fn as_input_hedra_character3audio_one_item_list(&self) -> Option<&Vec<InputHedraCharacter3AudioOneItem>> {
        match self {
                    Self::InputHedraCharacter3AudioOneItemList(value) => Some(value),
                    _ => None,
                }
    }

    pub fn into_input_hedra_character3audio_one_item_list(self) -> Option<Vec<InputHedraCharacter3AudioOneItem>> {
        match self {
                    Self::InputHedraCharacter3AudioOneItemList(value) => Some(value),
                    _ => None,
                }
    }
}

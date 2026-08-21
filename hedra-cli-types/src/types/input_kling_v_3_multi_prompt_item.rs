pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One shot of a published multi-shot storyboard.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct InputKlingV3MultiPromptItem {
    /// The prompt for this shot.
    #[serde(default)]
    pub prompt: String,
    /// Duration of this shot in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
}

impl InputKlingV3MultiPromptItem {
    pub fn builder() -> InputKlingV3MultiPromptItemBuilder {
        <InputKlingV3MultiPromptItemBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKlingV3MultiPromptItemBuilder {
    prompt: Option<String>,
    duration_ms: Option<i64>,
}

impl InputKlingV3MultiPromptItemBuilder {
    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKlingV3MultiPromptItem`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKlingV3MultiPromptItemBuilder::prompt)
    pub fn build(self) -> Result<InputKlingV3MultiPromptItem, BuildError> {
        Ok(InputKlingV3MultiPromptItem {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            duration_ms: self.duration_ms,
        })
    }
}

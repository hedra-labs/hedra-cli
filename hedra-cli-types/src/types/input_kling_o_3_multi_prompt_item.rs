pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// One shot of a published multi-shot storyboard.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct InputKlingO3MultiPromptItem {
    /// The prompt for this shot.
    #[serde(default)]
    pub prompt: String,
    /// Duration of this shot in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
}

impl InputKlingO3MultiPromptItem {
    pub fn builder() -> InputKlingO3MultiPromptItemBuilder {
        <InputKlingO3MultiPromptItemBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKlingO3MultiPromptItemBuilder {
    prompt: Option<String>,
    duration_ms: Option<i64>,
}

impl InputKlingO3MultiPromptItemBuilder {
    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKlingO3MultiPromptItem`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKlingO3MultiPromptItemBuilder::prompt)
    pub fn build(self) -> Result<InputKlingO3MultiPromptItem, BuildError> {
        Ok(InputKlingO3MultiPromptItem {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            duration_ms: self.duration_ms,
        })
    }
}

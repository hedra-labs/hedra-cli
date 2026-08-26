pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `elevenlabs-sound-effects`.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct InputElevenlabsSoundEffects {
    /// Generation prompt.
    #[serde(default)]
    pub text: String,
    /// Length of the generated sound, in milliseconds. Billed as whole seconds, rounded up. From 500 to 30000.
    #[serde(default)]
    pub duration_ms: i64,
    /// How literally the model reads the description; higher values track it more closely and lower values leave the model more latitude. From 0 to 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub prompt_influence: Option<f64>,
    /// Generate a sound that loops seamlessly end to end.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#loop: Option<bool>,
}

impl InputElevenlabsSoundEffects {
    pub fn builder() -> InputElevenlabsSoundEffectsBuilder {
        <InputElevenlabsSoundEffectsBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputElevenlabsSoundEffectsBuilder {
    text: Option<String>,
    duration_ms: Option<i64>,
    prompt_influence: Option<f64>,
    r#loop: Option<bool>,
}

impl InputElevenlabsSoundEffectsBuilder {
    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = Some(value.into());
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn prompt_influence(mut self, value: f64) -> Self {
        self.prompt_influence = Some(value);
        self
    }

    pub fn r#loop(mut self, value: bool) -> Self {
        self.r#loop = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputElevenlabsSoundEffects`].
    /// This method will fail if any of the following fields are not set:
    /// - [`text`](InputElevenlabsSoundEffectsBuilder::text)
    /// - [`duration_ms`](InputElevenlabsSoundEffectsBuilder::duration_ms)
    pub fn build(self) -> Result<InputElevenlabsSoundEffects, BuildError> {
        Ok(InputElevenlabsSoundEffects {
            text: self.text.ok_or_else(|| BuildError::missing_field("text"))?,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            prompt_influence: self.prompt_influence,
            r#loop: self.r#loop,
        })
    }
}

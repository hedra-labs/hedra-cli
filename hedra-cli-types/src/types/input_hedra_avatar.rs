pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `hedra-avatar`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputHedraAvatar {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputHedraAvatarAspectRatio,
    /// Output resolution.
    pub resolution: InputHedraAvatarResolution,
    /// Duration in ms. At most 600000.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// Start frame (image-to-video). At most 10.4 MB.
    pub start_image: InputHedraAvatarStartImage,
    /// Driving audio: a single reference, or a list of references for multi-speaker generation — one audio per speaker, played in list order. 1 to 4 audio files, each from 0.5s to 600s and at most 104.8 MB.
    pub audio: InputHedraAvatarAudio,
    /// Speaker position(s) in the start frame, as normalized [x, y] image coordinates (0-1 from the top-left).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box_target: Option<InputHedraAvatarBoundingBoxTarget>,
}

impl InputHedraAvatar {
    pub fn builder() -> InputHedraAvatarBuilder {
        <InputHedraAvatarBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputHedraAvatarBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputHedraAvatarAspectRatio>,
    resolution: Option<InputHedraAvatarResolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputHedraAvatarStartImage>,
    audio: Option<InputHedraAvatarAudio>,
    bounding_box_target: Option<InputHedraAvatarBoundingBoxTarget>,
}

impl InputHedraAvatarBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputHedraAvatarAspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputHedraAvatarResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputHedraAvatarStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn audio(mut self, value: InputHedraAvatarAudio) -> Self {
        self.audio = Some(value);
        self
    }

    pub fn bounding_box_target(mut self, value: InputHedraAvatarBoundingBoxTarget) -> Self {
        self.bounding_box_target = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputHedraAvatar`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputHedraAvatarBuilder::prompt)
    /// - [`aspect_ratio`](InputHedraAvatarBuilder::aspect_ratio)
    /// - [`resolution`](InputHedraAvatarBuilder::resolution)
    /// - [`start_image`](InputHedraAvatarBuilder::start_image)
    /// - [`audio`](InputHedraAvatarBuilder::audio)
    pub fn build(self) -> Result<InputHedraAvatar, BuildError> {
        Ok(InputHedraAvatar {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            duration_ms: self.duration_ms,
            start_image: self.start_image.ok_or_else(|| BuildError::missing_field("start_image"))?,
            audio: self.audio.ok_or_else(|| BuildError::missing_field("audio"))?,
            bounding_box_target: self.bounding_box_target,
        })
    }
}

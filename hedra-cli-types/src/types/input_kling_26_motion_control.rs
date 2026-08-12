pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-26-motion-control`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKling26MotionControl {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Whether the output character's orientation follows the reference video ('video' — better for complex motion) or the character image ('image' — better for camera movement). Also caps the source video: 30s for 'video', 10s for 'image'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_orientation: Option<InputKling26MotionControlCharacterOrientation>,
    /// Start frame (image-to-video).
    pub start_image: InputKling26MotionControlStartImage,
    /// Source video (video-to-video).
    pub source_video: InputKling26MotionControlSourceVideo,
    /// Output resolution to generate at.
    pub resolution: InputKling26MotionControlResolution,
}

impl InputKling26MotionControl {
    pub fn builder() -> InputKling26MotionControlBuilder {
        <InputKling26MotionControlBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKling26MotionControlBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    character_orientation: Option<InputKling26MotionControlCharacterOrientation>,
    start_image: Option<InputKling26MotionControlStartImage>,
    source_video: Option<InputKling26MotionControlSourceVideo>,
    resolution: Option<InputKling26MotionControlResolution>,
}

impl InputKling26MotionControlBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn character_orientation(mut self, value: InputKling26MotionControlCharacterOrientation) -> Self {
        self.character_orientation = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputKling26MotionControlStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputKling26MotionControlSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKling26MotionControlResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKling26MotionControl`].
    /// This method will fail if any of the following fields are not set:
    /// - [`start_image`](InputKling26MotionControlBuilder::start_image)
    /// - [`source_video`](InputKling26MotionControlBuilder::source_video)
    /// - [`resolution`](InputKling26MotionControlBuilder::resolution)
    pub fn build(self) -> Result<InputKling26MotionControl, BuildError> {
        Ok(InputKling26MotionControl {
            num_outputs: self.num_outputs,
            prompt: self.prompt,
            character_orientation: self.character_orientation,
            start_image: self.start_image.ok_or_else(|| BuildError::missing_field("start_image"))?,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
        })
    }
}

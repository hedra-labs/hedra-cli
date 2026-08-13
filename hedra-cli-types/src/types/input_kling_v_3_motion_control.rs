pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-v3-motion-control`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKlingV3MotionControl {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2500 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Whether the output character's orientation follows the reference video ('video' — better for complex motion) or the character image ('image' — better for camera movement). Also caps the source video: 30s for 'video', 10s for 'image'.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_orientation: Option<InputKlingV3MotionControlCharacterOrientation>,
    /// Start frame (image-to-video). From 340px to 3850px on each side and at most 10.4 MB.
    pub start_image: InputKlingV3MotionControlStartImage,
    /// Source video (video-to-video). From 3s to 30s, from 340px to 3850px on each side, and at most 104.8 MB.
    pub source_video: InputKlingV3MotionControlSourceVideo,
    /// Output resolution to generate at.
    pub resolution: InputKlingV3MotionControlResolution,
}

impl InputKlingV3MotionControl {
    pub fn builder() -> InputKlingV3MotionControlBuilder {
        <InputKlingV3MotionControlBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKlingV3MotionControlBuilder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    character_orientation: Option<InputKlingV3MotionControlCharacterOrientation>,
    start_image: Option<InputKlingV3MotionControlStartImage>,
    source_video: Option<InputKlingV3MotionControlSourceVideo>,
    resolution: Option<InputKlingV3MotionControlResolution>,
}

impl InputKlingV3MotionControlBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn character_orientation(mut self, value: InputKlingV3MotionControlCharacterOrientation) -> Self {
        self.character_orientation = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputKlingV3MotionControlStartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputKlingV3MotionControlSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKlingV3MotionControlResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKlingV3MotionControl`].
    /// This method will fail if any of the following fields are not set:
    /// - [`start_image`](InputKlingV3MotionControlBuilder::start_image)
    /// - [`source_video`](InputKlingV3MotionControlBuilder::source_video)
    /// - [`resolution`](InputKlingV3MotionControlBuilder::resolution)
    pub fn build(self) -> Result<InputKlingV3MotionControl, BuildError> {
        Ok(InputKlingV3MotionControl {
            num_outputs: self.num_outputs,
            prompt: self.prompt,
            character_orientation: self.character_orientation,
            start_image: self.start_image.ok_or_else(|| BuildError::missing_field("start_image"))?,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
        })
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `hedra-character-3`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputHedraCharacter3 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt.
    #[serde(default)]
    pub prompt: String,
    /// Output aspect ratio.
    pub aspect_ratio: InputHedraCharacter3AspectRatio,
    /// Output resolution.
    pub resolution: InputHedraCharacter3Resolution,
    /// Duration in ms. At most 600000.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// Start frame (image-to-video). At most 10.4 MB.
    pub start_image: InputHedraCharacter3StartImage,
    /// Driving audio: a single reference, or a list of references for multi-speaker generation — one audio per speaker, played in list order. 1 to 4 audio files, each from 0.5s to 600s and at most 104.8 MB.
    pub audio: InputHedraCharacter3Audio,
    /// Speaker position(s) in the start frame, as normalized [x, y] image coordinates (0-1 from the top-left).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box_target: Option<InputHedraCharacter3BoundingBoxTarget>,
}

impl InputHedraCharacter3 {
    pub fn builder() -> InputHedraCharacter3Builder {
        <InputHedraCharacter3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputHedraCharacter3Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    aspect_ratio: Option<InputHedraCharacter3AspectRatio>,
    resolution: Option<InputHedraCharacter3Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputHedraCharacter3StartImage>,
    audio: Option<InputHedraCharacter3Audio>,
    bounding_box_target: Option<InputHedraCharacter3BoundingBoxTarget>,
}

impl InputHedraCharacter3Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn aspect_ratio(mut self, value: InputHedraCharacter3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputHedraCharacter3Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputHedraCharacter3StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn audio(mut self, value: InputHedraCharacter3Audio) -> Self {
        self.audio = Some(value);
        self
    }

    pub fn bounding_box_target(mut self, value: InputHedraCharacter3BoundingBoxTarget) -> Self {
        self.bounding_box_target = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputHedraCharacter3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputHedraCharacter3Builder::prompt)
    /// - [`aspect_ratio`](InputHedraCharacter3Builder::aspect_ratio)
    /// - [`resolution`](InputHedraCharacter3Builder::resolution)
    /// - [`start_image`](InputHedraCharacter3Builder::start_image)
    /// - [`audio`](InputHedraCharacter3Builder::audio)
    pub fn build(self) -> Result<InputHedraCharacter3, BuildError> {
        Ok(InputHedraCharacter3 {
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

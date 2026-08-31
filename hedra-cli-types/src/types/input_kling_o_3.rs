pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `kling-o3`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, duration_ms, resolution, start_image; accepts quality: pro; resolution: 1080p | 4K
/// (2) requires: aspect_ratio, duration_ms, images, resolution; accepts quality: pro; resolution: 1080p | 4K
/// (3) requires: aspect_ratio, duration_ms, resolution; must omit: end_image, images, start_image; accepts quality: pro; resolution: 1080p | 4K
/// (4) requires: aspect_ratio, duration_ms, start_image; accepts quality: standard; resolution: 720p
/// (5) requires: aspect_ratio, duration_ms, images; accepts quality: standard; resolution: 720p
/// (6) requires: aspect_ratio, duration_ms; must omit: end_image, images, start_image; accepts quality: standard; resolution: 720p
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKlingO3 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. At most 2500 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Multi-shot storyboard; each shot carries its own prompt and duration. Shot durations must sum to at most 15000 ms, and the storyboard replaces `prompt` — supply one or the other, never both. A shot's prompt has no length limit, unlike the single `prompt` field. 1 to 6 items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_prompt: Option<Vec<InputKlingO3MultiPromptItem>>,
    /// How a multi-shot storyboard is cut. 'customize' honours each shot's declared duration; 'intelligent' lets the model determine the shot structure. Ignored unless `multi_prompt` is supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shot_type: Option<InputKlingO3ShotType>,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputKlingO3AspectRatio,
    /// Output resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<InputKlingO3Resolution>,
    /// Duration in ms.
    #[serde(default)]
    pub duration_ms: i64,
    /// Start frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputKlingO3StartImage>,
    /// End frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputKlingO3EndImage>,
    /// Reference images. 1 to 3 images, each at most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputKlingO3ImagesItem>>,
    /// Quality level to generate at. `standard` — the 720p tier. `pro` — the high-resolution tier, at 1080p and 4K.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputKlingO3Quality>,
}

impl InputKlingO3 {
    pub fn builder() -> InputKlingO3Builder {
        <InputKlingO3Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKlingO3Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    multi_prompt: Option<Vec<InputKlingO3MultiPromptItem>>,
    shot_type: Option<InputKlingO3ShotType>,
    generate_audio: Option<bool>,
    aspect_ratio: Option<InputKlingO3AspectRatio>,
    resolution: Option<InputKlingO3Resolution>,
    duration_ms: Option<i64>,
    start_image: Option<InputKlingO3StartImage>,
    end_image: Option<InputKlingO3EndImage>,
    images: Option<Vec<InputKlingO3ImagesItem>>,
    quality: Option<InputKlingO3Quality>,
}

impl InputKlingO3Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn multi_prompt(mut self, value: Vec<InputKlingO3MultiPromptItem>) -> Self {
        self.multi_prompt = Some(value);
        self
    }

    pub fn shot_type(mut self, value: InputKlingO3ShotType) -> Self {
        self.shot_type = Some(value);
        self
    }

    pub fn generate_audio(mut self, value: bool) -> Self {
        self.generate_audio = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputKlingO3AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputKlingO3Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputKlingO3StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputKlingO3EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputKlingO3ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn quality(mut self, value: InputKlingO3Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKlingO3`].
    /// This method will fail if any of the following fields are not set:
    /// - [`aspect_ratio`](InputKlingO3Builder::aspect_ratio)
    /// - [`duration_ms`](InputKlingO3Builder::duration_ms)
    pub fn build(self) -> Result<InputKlingO3, BuildError> {
        Ok(InputKlingO3 {
            num_outputs: self.num_outputs,
            prompt: self.prompt,
            multi_prompt: self.multi_prompt,
            shot_type: self.shot_type,
            generate_audio: self.generate_audio,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            resolution: self.resolution,
            duration_ms: self.duration_ms.ok_or_else(|| BuildError::missing_field("duration_ms"))?,
            start_image: self.start_image,
            end_image: self.end_image,
            images: self.images,
            quality: self.quality,
        })
    }
}

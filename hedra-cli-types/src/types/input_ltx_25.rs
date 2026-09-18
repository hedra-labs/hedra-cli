pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `ltx-2-5`.
/// 
/// Accepted field combinations (one per input mode):
/// (1) requires: aspect_ratio, audio, prompt; must omit: duration_ms, end_image, generate_audio; accepts resolution: 1080p
/// (2) requires: aspect_ratio, duration_ms, prompt, resolution, start_image; must omit: audio; accepts quality: fast
/// (3) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: audio, end_image, start_image; accepts aspect_ratio: 16:9 | 9:16; quality: fast
/// (4) requires: aspect_ratio, duration_ms, prompt, resolution, start_image; must omit: audio; accepts duration_ms: 6000 | 8000 | 10000; quality: pro; resolution: 1080p | 720p
/// (5) requires: aspect_ratio, duration_ms, prompt, resolution; must omit: audio, end_image, start_image; accepts aspect_ratio: 16:9 | 9:16; duration_ms: 6000 | 8000 | 10000; quality: pro; resolution: 1080p | 720p
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputLtx25 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Generation prompt. From 1 to 5000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Output resolution.
    pub resolution: InputLtx25Resolution,
    /// Output aspect ratio.
    pub aspect_ratio: InputLtx25AspectRatio,
    /// Driving audio. From 2s to 20s and at most 104.8 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<InputLtx25Audio>,
    /// Start frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_image: Option<InputLtx25StartImage>,
    /// Whether to generate native audio for the video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generate_audio: Option<bool>,
    /// Duration in ms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// End frame. At most 10.4 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_image: Option<InputLtx25EndImage>,
    /// Quality level to generate at. `fast` — tuned for turnaround, and the only level that renders 1440p, 2160p, or clips past 10 seconds. `pro` — the higher-fidelity tier, for final output at 720p or 1080p, and the level that caps both the clip it renders and the audio driving it at 10 seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputLtx25Quality>,
}

impl InputLtx25 {
    pub fn builder() -> InputLtx25Builder {
        <InputLtx25Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputLtx25Builder {
    num_outputs: Option<i64>,
    prompt: Option<String>,
    resolution: Option<InputLtx25Resolution>,
    aspect_ratio: Option<InputLtx25AspectRatio>,
    audio: Option<InputLtx25Audio>,
    start_image: Option<InputLtx25StartImage>,
    generate_audio: Option<bool>,
    duration_ms: Option<i64>,
    end_image: Option<InputLtx25EndImage>,
    quality: Option<InputLtx25Quality>,
}

impl InputLtx25Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn resolution(mut self, value: InputLtx25Resolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputLtx25AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn audio(mut self, value: InputLtx25Audio) -> Self {
        self.audio = Some(value);
        self
    }

    pub fn start_image(mut self, value: InputLtx25StartImage) -> Self {
        self.start_image = Some(value);
        self
    }

    pub fn generate_audio(mut self, value: bool) -> Self {
        self.generate_audio = Some(value);
        self
    }

    pub fn duration_ms(mut self, value: i64) -> Self {
        self.duration_ms = Some(value);
        self
    }

    pub fn end_image(mut self, value: InputLtx25EndImage) -> Self {
        self.end_image = Some(value);
        self
    }

    pub fn quality(mut self, value: InputLtx25Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputLtx25`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputLtx25Builder::prompt)
    /// - [`resolution`](InputLtx25Builder::resolution)
    /// - [`aspect_ratio`](InputLtx25Builder::aspect_ratio)
    pub fn build(self) -> Result<InputLtx25, BuildError> {
        Ok(InputLtx25 {
            num_outputs: self.num_outputs,
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            audio: self.audio,
            start_image: self.start_image,
            generate_audio: self.generate_audio,
            duration_ms: self.duration_ms,
            end_image: self.end_image,
            quality: self.quality,
        })
    }
}

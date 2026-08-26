pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `topaz-video-upscaler-starlight-precise`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputTopazVideoUpscalerStarlightPrecise {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// The video to upscale. At most 300s and at most 524.2 MB.
    pub source_video: InputTopazVideoUpscalerStarlightPreciseSourceVideo,
    /// Output resolution.
    pub resolution: InputTopazVideoUpscalerStarlightPreciseResolution,
    /// How far to soften the diffusion pass, for sources the default sharpening renders harshly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_sharpening: Option<InputTopazVideoUpscalerStarlightPreciseReduceSharpening>,
}

impl InputTopazVideoUpscalerStarlightPrecise {
    pub fn builder() -> InputTopazVideoUpscalerStarlightPreciseBuilder {
        <InputTopazVideoUpscalerStarlightPreciseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputTopazVideoUpscalerStarlightPreciseBuilder {
    num_outputs: Option<i64>,
    source_video: Option<InputTopazVideoUpscalerStarlightPreciseSourceVideo>,
    resolution: Option<InputTopazVideoUpscalerStarlightPreciseResolution>,
    reduce_sharpening: Option<InputTopazVideoUpscalerStarlightPreciseReduceSharpening>,
}

impl InputTopazVideoUpscalerStarlightPreciseBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputTopazVideoUpscalerStarlightPreciseSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputTopazVideoUpscalerStarlightPreciseResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn reduce_sharpening(mut self, value: InputTopazVideoUpscalerStarlightPreciseReduceSharpening) -> Self {
        self.reduce_sharpening = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputTopazVideoUpscalerStarlightPrecise`].
    /// This method will fail if any of the following fields are not set:
    /// - [`source_video`](InputTopazVideoUpscalerStarlightPreciseBuilder::source_video)
    /// - [`resolution`](InputTopazVideoUpscalerStarlightPreciseBuilder::resolution)
    pub fn build(self) -> Result<InputTopazVideoUpscalerStarlightPrecise, BuildError> {
        Ok(InputTopazVideoUpscalerStarlightPrecise {
            num_outputs: self.num_outputs,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            reduce_sharpening: self.reduce_sharpening,
        })
    }
}

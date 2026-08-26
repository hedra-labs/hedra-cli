pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `topaz-video-upscaler-starlight-fast`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputTopazVideoUpscalerStarlightFast {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// The video to upscale. At most 300s and at most 524.2 MB.
    pub source_video: InputTopazVideoUpscalerStarlightFastSourceVideo,
    /// Output resolution.
    pub resolution: InputTopazVideoUpscalerStarlightFastResolution,
    /// Soften the diffusion pass, which suits sources the default sharpening renders harshly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_sharpening: Option<bool>,
}

impl InputTopazVideoUpscalerStarlightFast {
    pub fn builder() -> InputTopazVideoUpscalerStarlightFastBuilder {
        <InputTopazVideoUpscalerStarlightFastBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputTopazVideoUpscalerStarlightFastBuilder {
    num_outputs: Option<i64>,
    source_video: Option<InputTopazVideoUpscalerStarlightFastSourceVideo>,
    resolution: Option<InputTopazVideoUpscalerStarlightFastResolution>,
    reduce_sharpening: Option<bool>,
}

impl InputTopazVideoUpscalerStarlightFastBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputTopazVideoUpscalerStarlightFastSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputTopazVideoUpscalerStarlightFastResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn reduce_sharpening(mut self, value: bool) -> Self {
        self.reduce_sharpening = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputTopazVideoUpscalerStarlightFast`].
    /// This method will fail if any of the following fields are not set:
    /// - [`source_video`](InputTopazVideoUpscalerStarlightFastBuilder::source_video)
    /// - [`resolution`](InputTopazVideoUpscalerStarlightFastBuilder::resolution)
    pub fn build(self) -> Result<InputTopazVideoUpscalerStarlightFast, BuildError> {
        Ok(InputTopazVideoUpscalerStarlightFast {
            num_outputs: self.num_outputs,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            reduce_sharpening: self.reduce_sharpening,
        })
    }
}

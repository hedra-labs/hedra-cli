pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `flux-3-video-upscaler-precise`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputFlux3VideoUpscalerPrecise {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// The video to upscale. At most 20s and at most 524.2 MB.
    pub source_video: InputFlux3VideoUpscalerPreciseSourceVideo,
    /// Output resolution.
    pub resolution: InputFlux3VideoUpscalerPreciseResolution,
}

impl InputFlux3VideoUpscalerPrecise {
    pub fn builder() -> InputFlux3VideoUpscalerPreciseBuilder {
        <InputFlux3VideoUpscalerPreciseBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputFlux3VideoUpscalerPreciseBuilder {
    num_outputs: Option<i64>,
    source_video: Option<InputFlux3VideoUpscalerPreciseSourceVideo>,
    resolution: Option<InputFlux3VideoUpscalerPreciseResolution>,
}

impl InputFlux3VideoUpscalerPreciseBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputFlux3VideoUpscalerPreciseSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputFlux3VideoUpscalerPreciseResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputFlux3VideoUpscalerPrecise`].
    /// This method will fail if any of the following fields are not set:
    /// - [`source_video`](InputFlux3VideoUpscalerPreciseBuilder::source_video)
    /// - [`resolution`](InputFlux3VideoUpscalerPreciseBuilder::resolution)
    pub fn build(self) -> Result<InputFlux3VideoUpscalerPrecise, BuildError> {
        Ok(InputFlux3VideoUpscalerPrecise {
            num_outputs: self.num_outputs,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
        })
    }
}

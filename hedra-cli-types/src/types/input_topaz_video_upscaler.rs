pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `topaz-video-upscaler`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputTopazVideoUpscaler {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// The video to upscale. At most 600s and at most 524.2 MB.
    pub source_video: InputTopazVideoUpscalerSourceVideo,
    /// Output resolution.
    pub resolution: InputTopazVideoUpscalerResolution,
    /// Interpolate the output to 60fps with this engine. Omit to keep the source rate; a source already at 60fps or faster keeps its own. 'high' doubles the charge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps_engine: Option<InputTopazVideoUpscalerFpsEngine>,
    /// Where the enhancement parameters come from: 'standard' uses Hedra's per-resolution presets, 'auto' lets Topaz estimate them from the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tuning: Option<InputTopazVideoUpscalerTuning>,
}

impl InputTopazVideoUpscaler {
    pub fn builder() -> InputTopazVideoUpscalerBuilder {
        <InputTopazVideoUpscalerBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputTopazVideoUpscalerBuilder {
    num_outputs: Option<i64>,
    source_video: Option<InputTopazVideoUpscalerSourceVideo>,
    resolution: Option<InputTopazVideoUpscalerResolution>,
    fps_engine: Option<InputTopazVideoUpscalerFpsEngine>,
    tuning: Option<InputTopazVideoUpscalerTuning>,
}

impl InputTopazVideoUpscalerBuilder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputTopazVideoUpscalerSourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    pub fn resolution(mut self, value: InputTopazVideoUpscalerResolution) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn fps_engine(mut self, value: InputTopazVideoUpscalerFpsEngine) -> Self {
        self.fps_engine = Some(value);
        self
    }

    pub fn tuning(mut self, value: InputTopazVideoUpscalerTuning) -> Self {
        self.tuning = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputTopazVideoUpscaler`].
    /// This method will fail if any of the following fields are not set:
    /// - [`source_video`](InputTopazVideoUpscalerBuilder::source_video)
    /// - [`resolution`](InputTopazVideoUpscalerBuilder::resolution)
    pub fn build(self) -> Result<InputTopazVideoUpscaler, BuildError> {
        Ok(InputTopazVideoUpscaler {
            num_outputs: self.num_outputs,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
            resolution: self.resolution.ok_or_else(|| BuildError::missing_field("resolution"))?,
            fps_engine: self.fps_engine,
            tuning: self.tuning,
        })
    }
}

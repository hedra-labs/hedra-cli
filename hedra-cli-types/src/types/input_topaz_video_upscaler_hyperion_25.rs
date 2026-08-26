pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `topaz-video-upscaler-hyperion-2-5`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputTopazVideoUpscalerHyperion25 {
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// The SDR video to convert. Its frames may hold at most 14,745,600 pixels (5120x2880). At most 600s and at most 524.2 MB.
    pub source_video: InputTopazVideoUpscalerHyperion25SourceVideo,
}

impl InputTopazVideoUpscalerHyperion25 {
    pub fn builder() -> InputTopazVideoUpscalerHyperion25Builder {
        <InputTopazVideoUpscalerHyperion25Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputTopazVideoUpscalerHyperion25Builder {
    num_outputs: Option<i64>,
    source_video: Option<InputTopazVideoUpscalerHyperion25SourceVideo>,
}

impl InputTopazVideoUpscalerHyperion25Builder {
    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn source_video(mut self, value: InputTopazVideoUpscalerHyperion25SourceVideo) -> Self {
        self.source_video = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputTopazVideoUpscalerHyperion25`].
    /// This method will fail if any of the following fields are not set:
    /// - [`source_video`](InputTopazVideoUpscalerHyperion25Builder::source_video)
    pub fn build(self) -> Result<InputTopazVideoUpscalerHyperion25, BuildError> {
        Ok(InputTopazVideoUpscalerHyperion25 {
            num_outputs: self.num_outputs,
            source_video: self.source_video.ok_or_else(|| BuildError::missing_field("source_video"))?,
        })
    }
}

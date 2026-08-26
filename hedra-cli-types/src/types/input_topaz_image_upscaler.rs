pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `topaz-image-upscaler`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputTopazImageUpscaler {
    /// The image to upscale. At most 30 MB.
    pub source_image: InputTopazImageUpscalerSourceImage,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Short edge of the upscaled image. The long edge follows the source's aspect ratio.
    pub target_resolution: InputTopazImageUpscalerTargetResolution,
    /// Restore facial detail, and how strongly. Off leaves faces to the general upscale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_recovery: Option<InputTopazImageUpscalerFaceRecovery>,
    /// How much a recovered face may be reimagined rather than reconstructed. Takes effect only with face_recovery on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub face_recovery_creativity: Option<InputTopazImageUpscalerFaceRecoveryCreativity>,
}

impl InputTopazImageUpscaler {
    pub fn builder() -> InputTopazImageUpscalerBuilder {
        <InputTopazImageUpscalerBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputTopazImageUpscalerBuilder {
    source_image: Option<InputTopazImageUpscalerSourceImage>,
    num_outputs: Option<i64>,
    target_resolution: Option<InputTopazImageUpscalerTargetResolution>,
    face_recovery: Option<InputTopazImageUpscalerFaceRecovery>,
    face_recovery_creativity: Option<InputTopazImageUpscalerFaceRecoveryCreativity>,
}

impl InputTopazImageUpscalerBuilder {
    pub fn source_image(mut self, value: InputTopazImageUpscalerSourceImage) -> Self {
        self.source_image = Some(value);
        self
    }

    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn target_resolution(mut self, value: InputTopazImageUpscalerTargetResolution) -> Self {
        self.target_resolution = Some(value);
        self
    }

    pub fn face_recovery(mut self, value: InputTopazImageUpscalerFaceRecovery) -> Self {
        self.face_recovery = Some(value);
        self
    }

    pub fn face_recovery_creativity(mut self, value: InputTopazImageUpscalerFaceRecoveryCreativity) -> Self {
        self.face_recovery_creativity = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputTopazImageUpscaler`].
    /// This method will fail if any of the following fields are not set:
    /// - [`source_image`](InputTopazImageUpscalerBuilder::source_image)
    /// - [`target_resolution`](InputTopazImageUpscalerBuilder::target_resolution)
    pub fn build(self) -> Result<InputTopazImageUpscaler, BuildError> {
        Ok(InputTopazImageUpscaler {
            source_image: self.source_image.ok_or_else(|| BuildError::missing_field("source_image"))?,
            num_outputs: self.num_outputs,
            target_resolution: self.target_resolution.ok_or_else(|| BuildError::missing_field("target_resolution"))?,
            face_recovery: self.face_recovery,
            face_recovery_creativity: self.face_recovery_creativity,
        })
    }
}

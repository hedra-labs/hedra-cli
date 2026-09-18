pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `topaz-image-upscaler-transparency`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputTopazImageUpscalerTransparency {
    /// The image to upscale. At most 30 MB.
    pub source_image: InputTopazImageUpscalerTransparencySourceImage,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
}

impl InputTopazImageUpscalerTransparency {
    pub fn builder() -> InputTopazImageUpscalerTransparencyBuilder {
        <InputTopazImageUpscalerTransparencyBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputTopazImageUpscalerTransparencyBuilder {
    source_image: Option<InputTopazImageUpscalerTransparencySourceImage>,
    num_outputs: Option<i64>,
}

impl InputTopazImageUpscalerTransparencyBuilder {
    pub fn source_image(mut self, value: InputTopazImageUpscalerTransparencySourceImage) -> Self {
        self.source_image = Some(value);
        self
    }

    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputTopazImageUpscalerTransparency`].
    /// This method will fail if any of the following fields are not set:
    /// - [`source_image`](InputTopazImageUpscalerTransparencyBuilder::source_image)
    pub fn build(self) -> Result<InputTopazImageUpscalerTransparency, BuildError> {
        Ok(InputTopazImageUpscalerTransparency {
            source_image: self.source_image.ok_or_else(|| BuildError::missing_field("source_image"))?,
            num_outputs: self.num_outputs,
        })
    }
}

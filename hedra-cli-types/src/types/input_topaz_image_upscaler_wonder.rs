pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `topaz-image-upscaler-wonder`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputTopazImageUpscalerWonder {
    /// The image to upscale. At most 30 MB.
    pub source_image: InputTopazImageUpscalerWonderSourceImage,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Short edge of the upscaled image. The long edge follows the source's aspect ratio.
    pub target_resolution: InputTopazImageUpscalerWonderTargetResolution,
    /// How far the generative pass may depart from the source. Lower values stay closer to the original pixels; higher values invent more detail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhancement_strength: Option<InputTopazImageUpscalerWonderEnhancementStrength>,
    /// Add film grain to the upscaled image, which hides the over-smooth look generative upscaling can leave.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub film_grain: Option<bool>,
}

impl InputTopazImageUpscalerWonder {
    pub fn builder() -> InputTopazImageUpscalerWonderBuilder {
        <InputTopazImageUpscalerWonderBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputTopazImageUpscalerWonderBuilder {
    source_image: Option<InputTopazImageUpscalerWonderSourceImage>,
    num_outputs: Option<i64>,
    target_resolution: Option<InputTopazImageUpscalerWonderTargetResolution>,
    enhancement_strength: Option<InputTopazImageUpscalerWonderEnhancementStrength>,
    film_grain: Option<bool>,
}

impl InputTopazImageUpscalerWonderBuilder {
    pub fn source_image(mut self, value: InputTopazImageUpscalerWonderSourceImage) -> Self {
        self.source_image = Some(value);
        self
    }

    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn target_resolution(mut self, value: InputTopazImageUpscalerWonderTargetResolution) -> Self {
        self.target_resolution = Some(value);
        self
    }

    pub fn enhancement_strength(mut self, value: InputTopazImageUpscalerWonderEnhancementStrength) -> Self {
        self.enhancement_strength = Some(value);
        self
    }

    pub fn film_grain(mut self, value: bool) -> Self {
        self.film_grain = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputTopazImageUpscalerWonder`].
    /// This method will fail if any of the following fields are not set:
    /// - [`source_image`](InputTopazImageUpscalerWonderBuilder::source_image)
    /// - [`target_resolution`](InputTopazImageUpscalerWonderBuilder::target_resolution)
    pub fn build(self) -> Result<InputTopazImageUpscalerWonder, BuildError> {
        Ok(InputTopazImageUpscalerWonder {
            source_image: self.source_image.ok_or_else(|| BuildError::missing_field("source_image"))?,
            num_outputs: self.num_outputs,
            target_resolution: self.target_resolution.ok_or_else(|| BuildError::missing_field("target_resolution"))?,
            enhancement_strength: self.enhancement_strength,
            film_grain: self.film_grain,
        })
    }
}

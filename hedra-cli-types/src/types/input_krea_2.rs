pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Model-specific inputs for `krea-2`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InputKrea2 {
    /// Generation prompt. From 1 to 5000 characters.
    #[serde(default)]
    pub prompt: String,
    /// Number of outputs generated per job. Only 1 is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_outputs: Option<i64>,
    /// Rewrite the prompt before generation. An LLM expands it into a fuller description and the model receives that text instead of the submitted one; the result's `prompt` reports what ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enhance_prompt: Option<bool>,
    /// Output aspect ratio.
    pub aspect_ratio: InputKrea2AspectRatio,
    /// How loosely the model may read the prompt. 'raw' stays closest to it; 'high' drifts furthest for a more inventive result. Omit for Krea's own default ('medium').
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creativity: Option<InputKrea2Creativity>,
    /// Reference images whose style — palette, lighting, texture, art direction — the generated image takes on. A reference with no clear subject of its own can crowd out the one you prompted for; lower `style_strength` to hold it back. Omit them to render from the prompt alone. 1 to 10 images, each at most 30 MB.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<InputKrea2ImagesItem>>,
    /// How strongly the reference images pull the render toward their style, applied to every one of them. Krea pulls as hard as it can unless you say otherwise; lower this to leave more of the prompt's own subject, and 0 to ignore the references' style entirely. From 0 to 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[serde(with = "crate::core::number_serializers::option")]
    pub style_strength: Option<f64>,
    /// Seed passed to Krea. It does not make a render repeatable — Krea returns a different image each time from the same seed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Quality level to generate at. `medium` — the smaller model, strongest on illustration, anime and painting. `large` — the larger model, for photorealism and the effects that sell it — motion blur, grain, low dynamic range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<InputKrea2Quality>,
}

impl InputKrea2 {
    pub fn builder() -> InputKrea2Builder {
        <InputKrea2Builder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct InputKrea2Builder {
    prompt: Option<String>,
    num_outputs: Option<i64>,
    enhance_prompt: Option<bool>,
    aspect_ratio: Option<InputKrea2AspectRatio>,
    creativity: Option<InputKrea2Creativity>,
    images: Option<Vec<InputKrea2ImagesItem>>,
    style_strength: Option<f64>,
    seed: Option<i64>,
    quality: Option<InputKrea2Quality>,
}

impl InputKrea2Builder {
    pub fn prompt(mut self, value: impl Into<String>) -> Self {
        self.prompt = Some(value.into());
        self
    }

    pub fn num_outputs(mut self, value: i64) -> Self {
        self.num_outputs = Some(value);
        self
    }

    pub fn enhance_prompt(mut self, value: bool) -> Self {
        self.enhance_prompt = Some(value);
        self
    }

    pub fn aspect_ratio(mut self, value: InputKrea2AspectRatio) -> Self {
        self.aspect_ratio = Some(value);
        self
    }

    pub fn creativity(mut self, value: InputKrea2Creativity) -> Self {
        self.creativity = Some(value);
        self
    }

    pub fn images(mut self, value: Vec<InputKrea2ImagesItem>) -> Self {
        self.images = Some(value);
        self
    }

    pub fn style_strength(mut self, value: f64) -> Self {
        self.style_strength = Some(value);
        self
    }

    pub fn seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    pub fn quality(mut self, value: InputKrea2Quality) -> Self {
        self.quality = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`InputKrea2`].
    /// This method will fail if any of the following fields are not set:
    /// - [`prompt`](InputKrea2Builder::prompt)
    /// - [`aspect_ratio`](InputKrea2Builder::aspect_ratio)
    pub fn build(self) -> Result<InputKrea2, BuildError> {
        Ok(InputKrea2 {
            prompt: self.prompt.ok_or_else(|| BuildError::missing_field("prompt"))?,
            num_outputs: self.num_outputs,
            enhance_prompt: self.enhance_prompt,
            aspect_ratio: self.aspect_ratio.ok_or_else(|| BuildError::missing_field("aspect_ratio"))?,
            creativity: self.creativity,
            images: self.images,
            style_strength: self.style_strength,
            seed: self.seed,
            quality: self.quality,
        })
    }
}

pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct AiModel {
    /// ID of the model
    #[serde(default)]
    pub id: String,
    /// Name of the model
    #[serde(default)]
    pub name: String,
    /// Description of the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Type of generation the model applies to.
    #[serde(default)]
    pub r#type: String,
    /// Aspect ratios the model supports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspect_ratios: Option<Vec<String>>,
    /// Resolutions the model supports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolutions: Option<Vec<String>>,
    /// Durations the model supports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub durations: Option<Vec<String>>,
    /// Whether the model is conditioned by a start frame.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_start_frame: Option<bool>,
    /// Whether the model is conditioned by an end frame.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_end_frame: Option<bool>,
    /// Whether the model is conditioned by audio input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_audio_input: Option<bool>,
    /// Whether the model requires video input (video-to-video).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_input_video: Option<bool>,
    /// Whether the model requires character orientation (motion control).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires_character_orientation: Option<bool>,
    /// Maximum video duration in milliseconds. Only applies to audio-driven models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_ms: Option<i64>,
    /// Whether the model supports custom resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_resolution: Option<bool>,
    /// Pricing details of the model.
    #[serde(default)]
    pub price_details: AiModelPrice,
    /// Extensible pricing information with dimension modifiers for resolution, audio, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<Pricing>,
    /// Width and height for each aspect_ratio and resolution tuple.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<HashMap<String, Option<HashMap<String, Option<Dimension>>>>>,
    /// URL of the model's logo in SVG format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    /// Whether this is a premium model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub premium: Option<bool>,
    /// Display order for UI sorting. Lower values appear first.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_order: Option<i64>,
}

impl AiModel {
    pub fn builder() -> AiModelBuilder {
        <AiModelBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct AiModelBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    r#type: Option<String>,
    aspect_ratios: Option<Vec<String>>,
    resolutions: Option<Vec<String>>,
    durations: Option<Vec<String>>,
    requires_start_frame: Option<bool>,
    requires_end_frame: Option<bool>,
    requires_audio_input: Option<bool>,
    requires_input_video: Option<bool>,
    requires_character_orientation: Option<bool>,
    max_duration_ms: Option<i64>,
    custom_resolution: Option<bool>,
    price_details: Option<AiModelPrice>,
    pricing: Option<Pricing>,
    dimensions: Option<HashMap<String, Option<HashMap<String, Option<Dimension>>>>>,
    logo_url: Option<String>,
    premium: Option<bool>,
    display_order: Option<i64>,
}

impl AiModelBuilder {
    pub fn id(mut self, value: impl Into<String>) -> Self {
        self.id = Some(value.into());
        self
    }

    pub fn name(mut self, value: impl Into<String>) -> Self {
        self.name = Some(value.into());
        self
    }

    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    pub fn r#type(mut self, value: impl Into<String>) -> Self {
        self.r#type = Some(value.into());
        self
    }

    pub fn aspect_ratios(mut self, value: Vec<String>) -> Self {
        self.aspect_ratios = Some(value);
        self
    }

    pub fn resolutions(mut self, value: Vec<String>) -> Self {
        self.resolutions = Some(value);
        self
    }

    pub fn durations(mut self, value: Vec<String>) -> Self {
        self.durations = Some(value);
        self
    }

    pub fn requires_start_frame(mut self, value: bool) -> Self {
        self.requires_start_frame = Some(value);
        self
    }

    pub fn requires_end_frame(mut self, value: bool) -> Self {
        self.requires_end_frame = Some(value);
        self
    }

    pub fn requires_audio_input(mut self, value: bool) -> Self {
        self.requires_audio_input = Some(value);
        self
    }

    pub fn requires_input_video(mut self, value: bool) -> Self {
        self.requires_input_video = Some(value);
        self
    }

    pub fn requires_character_orientation(mut self, value: bool) -> Self {
        self.requires_character_orientation = Some(value);
        self
    }

    pub fn max_duration_ms(mut self, value: i64) -> Self {
        self.max_duration_ms = Some(value);
        self
    }

    pub fn custom_resolution(mut self, value: bool) -> Self {
        self.custom_resolution = Some(value);
        self
    }

    pub fn price_details(mut self, value: AiModelPrice) -> Self {
        self.price_details = Some(value);
        self
    }

    pub fn pricing(mut self, value: Pricing) -> Self {
        self.pricing = Some(value);
        self
    }

    pub fn dimensions(mut self, value: HashMap<String, Option<HashMap<String, Option<Dimension>>>>) -> Self {
        self.dimensions = Some(value);
        self
    }

    pub fn logo_url(mut self, value: impl Into<String>) -> Self {
        self.logo_url = Some(value.into());
        self
    }

    pub fn premium(mut self, value: bool) -> Self {
        self.premium = Some(value);
        self
    }

    pub fn display_order(mut self, value: i64) -> Self {
        self.display_order = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`AiModel`].
    /// This method will fail if any of the following fields are not set:
    /// - [`id`](AiModelBuilder::id)
    /// - [`name`](AiModelBuilder::name)
    /// - [`r#type`](AiModelBuilder::r#type)
    /// - [`price_details`](AiModelBuilder::price_details)
    pub fn build(self) -> Result<AiModel, BuildError> {
        Ok(AiModel {
            id: self.id.ok_or_else(|| BuildError::missing_field("id"))?,
            name: self.name.ok_or_else(|| BuildError::missing_field("name"))?,
            description: self.description,
            r#type: self.r#type.ok_or_else(|| BuildError::missing_field("r#type"))?,
            aspect_ratios: self.aspect_ratios,
            resolutions: self.resolutions,
            durations: self.durations,
            requires_start_frame: self.requires_start_frame,
            requires_end_frame: self.requires_end_frame,
            requires_audio_input: self.requires_audio_input,
            requires_input_video: self.requires_input_video,
            requires_character_orientation: self.requires_character_orientation,
            max_duration_ms: self.max_duration_ms,
            custom_resolution: self.custom_resolution,
            price_details: self.price_details.ok_or_else(|| BuildError::missing_field("price_details"))?,
            pricing: self.pricing,
            dimensions: self.dimensions,
            logo_url: self.logo_url,
            premium: self.premium,
            display_order: self.display_order,
        })
    }
}

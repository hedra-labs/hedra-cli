pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Element for character tracking in Kling O1 Edit.
/// 
/// Reference as @Element1, @Element2, etc. in prompt.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct KlingO1EditElement {
    /// ID of the frontal image asset for this element.
    #[serde(default)]
    pub frontal_image_asset_id: String,
    /// Optional IDs of additional reference image assets (different angles).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_image_asset_ids: Option<Vec<String>>,
}

impl KlingO1EditElement {
    pub fn builder() -> KlingO1EditElementBuilder {
        <KlingO1EditElementBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct KlingO1EditElementBuilder {
    frontal_image_asset_id: Option<String>,
    reference_image_asset_ids: Option<Vec<String>>,
}

impl KlingO1EditElementBuilder {
    pub fn frontal_image_asset_id(mut self, value: impl Into<String>) -> Self {
        self.frontal_image_asset_id = Some(value.into());
        self
    }

    pub fn reference_image_asset_ids(mut self, value: Vec<String>) -> Self {
        self.reference_image_asset_ids = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`KlingO1EditElement`].
    /// This method will fail if any of the following fields are not set:
    /// - [`frontal_image_asset_id`](KlingO1EditElementBuilder::frontal_image_asset_id)
    pub fn build(self) -> Result<KlingO1EditElement, BuildError> {
        Ok(KlingO1EditElement {
            frontal_image_asset_id: self.frontal_image_asset_id.ok_or_else(|| BuildError::missing_field("frontal_image_asset_id"))?,
            reference_image_asset_ids: self.reference_image_asset_ids,
        })
    }
}

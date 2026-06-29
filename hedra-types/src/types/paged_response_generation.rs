pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PagedResponseGeneration {
    /// Paging information.
    #[serde(default)]
    pub page_info: PageInfo,
    /// Page data.
    #[serde(default)]
    pub data: Vec<Generation>,
}

impl PagedResponseGeneration {
    pub fn builder() -> PagedResponseGenerationBuilder {
        <PagedResponseGenerationBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct PagedResponseGenerationBuilder {
    page_info: Option<PageInfo>,
    data: Option<Vec<Generation>>,
}

impl PagedResponseGenerationBuilder {
    pub fn page_info(mut self, value: PageInfo) -> Self {
        self.page_info = Some(value);
        self
    }

    pub fn data(mut self, value: Vec<Generation>) -> Self {
        self.data = Some(value);
        self
    }

    /// Consumes the builder and constructs a [`PagedResponseGeneration`].
    /// This method will fail if any of the following fields are not set:
    /// - [`page_info`](PagedResponseGenerationBuilder::page_info)
    /// - [`data`](PagedResponseGenerationBuilder::data)
    pub fn build(self) -> Result<PagedResponseGeneration, BuildError> {
        Ok(PagedResponseGeneration {
            page_info: self.page_info.ok_or_else(|| BuildError::missing_field("page_info"))?,
            data: self.data.ok_or_else(|| BuildError::missing_field("data"))?,
        })
    }
}

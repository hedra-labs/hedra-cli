pub use crate::prelude::*;
#[allow(unused_imports)]
use super::*;

/// Query parameters for searchVoices
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq, Hash)]
pub struct SearchVoicesQueryRequest {
    /// What the voice should sound like, in plain words — "warm british narrator", "energetic young announcer". Matched against the whole library for this model's provider, not just the voices `GET /v3/models/{model}/voices` returns.
    #[serde(default)]
    pub q: String,
    /// Maximum voices to return. Applies to the whole response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// Only voices curated with this gender.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<VoiceGender>,
    /// Only voices curated for this language, as an ISO 639-1 two-letter code (`en`, `es`, `fr`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

impl SearchVoicesQueryRequest {
    pub fn builder() -> SearchVoicesQueryRequestBuilder {
        <SearchVoicesQueryRequestBuilder as Default>::default()
    }
}

#[derive(Clone, PartialEq, Default, Debug)]
#[non_exhaustive]
pub struct SearchVoicesQueryRequestBuilder {
    q: Option<String>,
    limit: Option<i64>,
    gender: Option<VoiceGender>,
    language: Option<String>,
}

impl SearchVoicesQueryRequestBuilder {
    pub fn q(mut self, value: impl Into<String>) -> Self {
        self.q = Some(value.into());
        self
    }

    pub fn limit(mut self, value: i64) -> Self {
        self.limit = Some(value);
        self
    }

    pub fn gender(mut self, value: VoiceGender) -> Self {
        self.gender = Some(value);
        self
    }

    pub fn language(mut self, value: impl Into<String>) -> Self {
        self.language = Some(value.into());
        self
    }

    /// Consumes the builder and constructs a [`SearchVoicesQueryRequest`].
    /// This method will fail if any of the following fields are not set:
    /// - [`q`](SearchVoicesQueryRequestBuilder::q)
    pub fn build(self) -> Result<SearchVoicesQueryRequest, BuildError> {
        Ok(SearchVoicesQueryRequest {
            q: self.q.ok_or_else(|| BuildError::missing_field("q"))?,
            limit: self.limit,
            gender: self.gender,
            language: self.language,
        })
    }
}


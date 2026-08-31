use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions};
use reqwest::Method;
use std::collections::HashMap;

pub struct ModelsClient {
    pub http_client: HttpClient,
}

impl ModelsClient {
    pub fn new(config: ClientConfig) -> Result<Self, ApiError> {
        Ok(Self {
            http_client: HttpClient::new(config.clone())?,
        })
    }

    /// # Examples
    ///
    /// ```no_run
    /// use hedra_cli_sdk::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = ClientConfig {
    ///         token: Some("<token>".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let client = HedraCliClient::new(config).expect("Failed to build client");
    ///     client
    ///         .models
    ///         .list(
    ///             &ModelsListQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn list(
        &self,
        request: &ModelsListQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<ModelListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.15.5".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                "models",
                None,
                QueryBuilder::new()
                    .serialize("modality", request.modality.clone())
                    .build(),
                options,
            )
            .await
    }

    /// # Examples
    ///
    /// ```no_run
    /// use hedra_cli_sdk::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = ClientConfig {
    ///         token: Some("<token>".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let client = HedraCliClient::new(config).expect("Failed to build client");
    ///     client.models.get(&"model".to_string(), None).await;
    /// }
    /// ```
    pub async fn get(
        &self,
        model: &str,
        options: Option<RequestOptions>,
    ) -> Result<ModelDetail, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.15.5".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("models/{}", model),
                None,
                None,
                options,
            )
            .await
    }

    /// # Examples
    ///
    /// ```no_run
    /// use hedra_cli_sdk::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = ClientConfig {
    ///         token: Some("<token>".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let client = HedraCliClient::new(config).expect("Failed to build client");
    ///     client
    ///         .models
    ///         .list_model_jobs(
    ///             &"model".to_string(),
    ///             &ListModelJobsQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn list_model_jobs(
        &self,
        model: &str,
        request: &ListModelJobsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<JobListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.15.5".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("models/{}/jobs", model),
                None,
                QueryBuilder::new()
                    .int("limit", request.limit.clone())
                    .serialize("cursor", request.cursor.clone())
                    .build(),
                options,
            )
            .await
    }

    /// Voices this model accepts — the shared library, plus the caller's own cloned voices when the request carries credentials.
    ///
    /// # Arguments
    ///
    /// * `model` - The model's public id (`GET /v3/models`).
    /// * `options` - Additional request options such as headers, timeout, etc.
    ///
    /// # Returns
    ///
    /// JSON response from the API
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hedra_cli_sdk::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = ClientConfig {
    ///         token: Some("<token>".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let client = HedraCliClient::new(config).expect("Failed to build client");
    ///     client.models.list_voices(&"model".to_string(), None).await;
    /// }
    /// ```
    pub async fn list_voices(
        &self,
        model: &str,
        options: Option<RequestOptions>,
    ) -> Result<VoiceListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.15.5".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("models/{}/voices", model),
                None,
                None,
                options,
            )
            .await
    }

    /// The voices this model accepts, ranked against a description — the whole shared library, including the voices the listing does not return.
    ///
    /// # Arguments
    ///
    /// * `model` - The model's public id (`GET /v3/models`).
    /// * `q` - What the voice should sound like, in plain words — "warm british narrator", "energetic young announcer". Matched against the whole library for this model's provider, not just the voices `GET /v3/models/{model}/voices` returns.
    /// * `limit` - Maximum voices to return. Applies to the whole response.
    /// * `gender` - Only voices curated with this gender.
    /// * `language` - Only voices curated for this language, as an ISO 639-1 two-letter code (`en`, `es`, `fr`).
    /// * `options` - Additional request options such as headers, timeout, etc.
    ///
    /// # Returns
    ///
    /// JSON response from the API
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hedra_cli_sdk::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = ClientConfig {
    ///         token: Some("<token>".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let client = HedraCliClient::new(config).expect("Failed to build client");
    ///     client
    ///         .models
    ///         .search_voices(
    ///             &"model".to_string(),
    ///             &SearchVoicesQueryRequest {
    ///                 q: "q".to_string(),
    ///                 limit: None,
    ///                 gender: None,
    ///                 language: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn search_voices(
        &self,
        model: &str,
        request: &SearchVoicesQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<VoiceListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.15.5".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("models/{}/voices/search", model),
                None,
                QueryBuilder::new()
                    .string("q", request.q.clone())
                    .int("limit", request.limit.clone())
                    .serialize("gender", request.gender.clone())
                    .serialize("language", request.language.clone())
                    .build(),
                options,
            )
            .await
    }

    /// A standalone one-operation OpenAPI spec for this model's submit call.
    ///
    /// # Arguments
    ///
    /// * `model` - The model's public id (`GET /v3/models`).
    /// * `options` - Additional request options such as headers, timeout, etc.
    ///
    /// # Returns
    ///
    /// JSON response from the API
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hedra_cli_sdk::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = ClientConfig {
    ///         token: Some("<token>".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let client = HedraCliClient::new(config).expect("Failed to build client");
    ///     client.models.get_openapi(&"model".to_string(), None).await;
    /// }
    /// ```
    pub async fn get_openapi(
        &self,
        model: &str,
        options: Option<RequestOptions>,
    ) -> Result<HashMap<String, serde_json::Value>, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.15.5".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("models/{}/openapi.json", model),
                None,
                None,
                options,
            )
            .await
    }

    /// # Examples
    ///
    /// ```no_run
    /// use hedra_cli_sdk::prelude::*;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let config = ClientConfig {
    ///         token: Some("<token>".to_string()),
    ///         ..Default::default()
    ///     };
    ///     let client = HedraCliClient::new(config).expect("Failed to build client");
    ///     client
    ///         .models
    ///         .estimate(
    ///             &"model".to_string(),
    ///             &EstimateRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn estimate(
        &self,
        model: &str,
        request: &EstimateRequest,
        options: Option<RequestOptions>,
    ) -> Result<EstimateResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.15.5".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                &format!("models/{}/estimate", model),
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }
}

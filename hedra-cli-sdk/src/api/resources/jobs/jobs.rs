use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions, SseStream};
use reqwest::Method;

pub struct JobsClient {
    pub http_client: HttpClient,
}

impl JobsClient {
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
    ///         .jobs
    ///         .list(
    ///             &JobsListQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn list(
        &self,
        request: &JobsListQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<JobListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                "jobs",
                None,
                QueryBuilder::new()
                    .int("limit", request.limit.clone())
                    .serialize("cursor", request.cursor.clone())
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
    ///     client.jobs.get(&"job_id".to_string(), None).await;
    /// }
    /// ```
    pub async fn get(
        &self,
        job_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<ResultResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("jobs/{}", job_id),
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
    ///         .jobs
    ///         .get_status(
    ///             &"job_id".to_string(),
    ///             &GetStatusQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn get_status(
        &self,
        job_id: &str,
        request: &GetStatusQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<StatusResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("jobs/{}/status", job_id),
                None,
                QueryBuilder::new()
                    .serialize("logs_after", request.logs_after.clone())
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
    ///     client
    ///         .jobs
    ///         .list_job_logs(
    ///             &"job_id".to_string(),
    ///             &ListJobLogsQueryRequest {
    ///                 ..Default::default()
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn list_job_logs(
        &self,
        job_id: &str,
        request: &ListJobLogsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<JobLogListResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::GET,
                &format!("jobs/{}/logs", job_id),
                None,
                QueryBuilder::new()
                    .int("limit", request.limit.clone())
                    .serialize("cursor", request.cursor.clone())
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
    ///     client.jobs.stream(&"job_id".to_string(), None).await;
    /// }
    /// ```
    pub async fn stream(
        &self,
        job_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<SseStream<JobsStreamResponse>, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_sse_request(
                Method::GET,
                &format!("jobs/{}/stream", job_id),
                None,
                None,
                options,
                Some("[STREAM_DONE]".to_string()),
            )
            .await
    }

    /// Ultra high quality generations for professional grade images.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_dreamina31(
    ///             &SubmitBodyDreamina31 {
    ///                 input: InputDreamina31 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputDreamina31AspectRatio::Sixteen9,
    ///                     resolution: InputDreamina31Resolution::FiveHundredFortyP,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_dreamina31(
        &self,
        request: &SubmitBodyDreamina31,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/dreamina-31",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_elevenlabs_flash_multilingual_v2(
    ///             &SubmitBodyElevenlabsFlashMultilingualV2 {
    ///                 input: InputElevenlabsFlashMultilingualV2 {
    ///                     text: "text".to_string(),
    ///                     voice_id: "voice_id".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_elevenlabs_flash_multilingual_v2(
        &self,
        request: &SubmitBodyElevenlabsFlashMultilingualV2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/elevenlabs-flash-multilingual-v2",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_elevenlabs_flash_v2(
    ///             &SubmitBodyElevenlabsFlashV2 {
    ///                 input: InputElevenlabsFlashV2 {
    ///                     text: "text".to_string(),
    ///                     voice_id: "voice_id".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_elevenlabs_flash_v2(
        &self,
        request: &SubmitBodyElevenlabsFlashV2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/elevenlabs-flash-v2",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_elevenlabs_multilingual_v2(
    ///             &SubmitBodyElevenlabsMultilingualV2 {
    ///                 input: InputElevenlabsMultilingualV2 {
    ///                     text: "text".to_string(),
    ///                     voice_id: "voice_id".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_elevenlabs_multilingual_v2(
        &self,
        request: &SubmitBodyElevenlabsMultilingualV2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/elevenlabs-multilingual-v2",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// ElevenLabs V3
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_elevenlabs_v3(
    ///             &SubmitBodyElevenlabsV3 {
    ///                 input: InputElevenlabsV3 {
    ///                     text: "text".to_string(),
    ///                     voice_id: "voice_id".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_elevenlabs_v3(
        &self,
        request: &SubmitBodyElevenlabsV3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/elevenlabs-v3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Premium color depth and clarity when you want campaign-ready art that feels handcrafted.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_flux11pro(
    ///             &SubmitBodyFlux11Pro {
    ///                 input: InputFlux11Pro {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputFlux11ProAspectRatio::One1,
    ///                     resolution: InputFlux11ProResolution::FiveHundredFortyP,
    ///                     output_format: None,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_flux11pro(
        &self,
        request: &SubmitBodyFlux11Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/flux-11-pro",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// The big-canvas choice for ultra-high-res images and flagship visuals.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_flux11ultra(
    ///             &SubmitBodyFlux11Ultra {
    ///                 input: InputFlux11Ultra {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputFlux11UltraAspectRatio::Sixteen9,
    ///                     resolution: None,
    ///                     output_format: None,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_flux11ultra(
        &self,
        request: &SubmitBodyFlux11Ultra,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/flux-11-ultra",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Black Forest Labs FLUX.3 text-to-video with native audio.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_flux3(
    ///             &SubmitBodyFlux3 {
    ///                 input: InputFlux3 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputFlux3AspectRatio::Auto,
    ///                     resolution: InputFlux3Resolution::SevenHundredTwentyP,
    ///                     duration_ms: 1,
    ///                     generate_audio: None,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_flux3(
        &self,
        request: &SubmitBodyFlux3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/flux-3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Fast and light for quick concepts or high-volume social posts on a budget.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_flux_dev(
    ///             &SubmitBodyFluxDev {
    ///                 input: InputFluxDev {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputFluxDevAspectRatio::One1,
    ///                     resolution: InputFluxDevResolution::FiveHundredFortyP,
    ///                     output_format: None,
    ///                     seed: None,
    ///                     guidance_scale: None,
    ///                     num_inference_steps: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_flux_dev(
        &self,
        request: &SubmitBodyFluxDev,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/flux-dev",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Highest-fidelity reference-image support for complex, multi-element scenes and perfectly matched branded visuals.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_flux_kontext_max(
    ///             &SubmitBodyFluxKontextMax {
    ///                 input: InputFluxKontextMax {
    ///                     prompt: "prompt".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_flux_kontext_max(
        &self,
        request: &SubmitBodyFluxKontextMax,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/flux-kontext-max",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Reference-image support for character, brand, or style consistency.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_flux_kontext_pro(
    ///             &SubmitBodyFluxKontextPro {
    ///                 input: InputFluxKontextPro {
    ///                     prompt: "prompt".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_flux_kontext_pro(
        &self,
        request: &SubmitBodyFluxKontextPro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/flux-kontext-pro",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Image creation and editing with FLUX.2 [flex] from Black Forest Labs.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_flux2flex(
    ///             &SubmitBodyFlux2Flex {
    ///                 input: InputFlux2Flex {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputFlux2FlexAspectRatio::Sixteen9,
    ///                     images: None,
    ///                     output_format: None,
    ///                     seed: None,
    ///                     guidance: None,
    ///                     steps: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_flux2flex(
        &self,
        request: &SubmitBodyFlux2Flex,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/flux2-flex",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Flux.2 [klein] 9B model from Black Forest Labs.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_flux2klein9b(
    ///             &SubmitBodyFlux2Klein9B {
    ///                 input: InputFlux2Klein9B {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputFlux2Klein9BAspectRatio::Sixteen9,
    ///                     images: None,
    ///                     output_format: None,
    ///                     negative_prompt: None,
    ///                     seed: None,
    ///                     guidance_scale: None,
    ///                     num_inference_steps: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_flux2klein9b(
        &self,
        request: &SubmitBodyFlux2Klein9B,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/flux2-klein-9b",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// FLUX.2 [max] delivers state-of-the-art image generation and advanced image editing with exceptional realism, precision, and consistency.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_flux2max(
    ///             &SubmitBodyFlux2Max {
    ///                 input: InputFlux2Max {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputFlux2MaxAspectRatio::Sixteen9,
    ///                     images: None,
    ///                     output_format: None,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_flux2max(
        &self,
        request: &SubmitBodyFlux2Max,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/flux2-max",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Image creation and editing with FLUX.2 [pro] from Black Forest Labs. Ideal for high-quality image manipulation, style transfer, and sequential editing workflows
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_flux2pro(
    ///             &SubmitBodyFlux2Pro {
    ///                 input: InputFlux2Pro {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputFlux2ProAspectRatio::Sixteen9,
    ///                     images: None,
    ///                     output_format: None,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_flux2pro(
        &self,
        request: &SubmitBodyFlux2Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/flux2-pro",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Gemini's fast multimodal video model — cinematic clips with native audio from a prompt, a keyframe, or reference images.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_gemini_omni_flash(
    ///             &SubmitBodyGeminiOmniFlash {
    ///                 input: InputGeminiOmniFlash {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputGeminiOmniFlashAspectRatio::Sixteen9,
    ///                     resolution: None,
    ///                     duration_ms: None,
    ///                     start_image: None,
    ///                     source_video: None,
    ///                     images: None,
    ///                     videos: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_gemini_omni_flash(
        &self,
        request: &SubmitBodyGeminiOmniFlash,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/gemini-omni-flash",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// OpenAI-powered image generation with exceptional prompt understanding and versatile editing capabilities.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_gpt_image15(
    ///             &SubmitBodyGptImage15 {
    ///                 input: InputGptImage15 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputGptImage15AspectRatio::One1,
    ///                     resolution: None,
    ///                     images: None,
    ///                     output_format: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_gpt_image15(
        &self,
        request: &SubmitBodyGptImage15,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/gpt-image-15",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// OpenAI's balanced tier; moderate cost and fidelity, ideal for iterative refinement and everyday generation.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_gpt_image2(
    ///             &SubmitBodyGptImage2 {
    ///                 input: InputGptImage2 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputGptImage2AspectRatio::Sixteen9,
    ///                     resolution: InputGptImage2Resolution::OneK,
    ///                     images: None,
    ///                     output_format: None,
    ///                     quality: InputGptImage2Quality::Low,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_gpt_image2(
        &self,
        request: &SubmitBodyGptImage2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/gpt-image-2",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// xAI's Grok Imagine image generation model
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_grok_imagine(
    ///             &SubmitBodyGrokImagine {
    ///                 input: InputGrokImagine {
    ///                     prompt: "prompt".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_grok_imagine(
        &self,
        request: &SubmitBodyGrokImagine,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/grok-imagine",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// xAI's text-to-video generation model.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_grok_video(
    ///             &SubmitBodyGrokVideo {
    ///                 input: InputGrokVideo {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputGrokVideoAspectRatio::Auto,
    ///                     resolution: InputGrokVideoResolution::FourHundredEightyP,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_grok_video(
        &self,
        request: &SubmitBodyGrokVideo,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/grok-video",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Generate video from text with Alibaba Happy Horse 1.0.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_happy_horse(
    ///             &SubmitBodyHappyHorse {
    ///                 input: InputHappyHorse {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputHappyHorseAspectRatio::TwentyOne9,
    ///                     resolution: InputHappyHorseResolution::SevenHundredTwentyP,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     images: None,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_happy_horse(
        &self,
        request: &SubmitBodyHappyHorse,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/happy-horse",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Hedra's latest longform avatar model, audio to video will full multi-language support. Perfect for talking and singing video with speaker selection up to 10 minutes long.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_hedra_avatar(
    ///             &SubmitBodyHedraAvatar {
    ///                 input: InputHedraAvatar {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputHedraAvatarAspectRatio::One1,
    ///                     resolution: InputHedraAvatarResolution::FiveHundredFortyP,
    ///                     duration_ms: None,
    ///                     start_image: InputHedraAvatarStartImage::URL {
    ///                         data: InputHedraAvatarStartImageURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     audio: InputHedraAvatarAudio::InputHedraAvatarAudioZero(
    ///                         InputHedraAvatarAudioZero::URL {
    ///                             data: InputHedraAvatarAudioZeroURL {
    ///                                 url: "url".to_string(),
    ///                                 ..Default::default()
    ///                             },
    ///                         },
    ///                     ),
    ///                     bounding_box_target: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_hedra_avatar(
        &self,
        request: &SubmitBodyHedraAvatar,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/hedra-avatar",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Hedra's latest longform avatar model, audio to video will full multi-language support. Perfect for talking and singing video with speaker selection up to 10 minutes long.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_hedra_character3(
    ///             &SubmitBodyHedraCharacter3 {
    ///                 input: InputHedraCharacter3 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputHedraCharacter3AspectRatio::One1,
    ///                     resolution: InputHedraCharacter3Resolution::FiveHundredFortyP,
    ///                     duration_ms: None,
    ///                     start_image: InputHedraCharacter3StartImage::URL {
    ///                         data: InputHedraCharacter3StartImageURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     audio: InputHedraCharacter3Audio::InputHedraCharacter3AudioZero(
    ///                         InputHedraCharacter3AudioZero::URL {
    ///                             data: InputHedraCharacter3AudioZeroURL {
    ///                                 url: "url".to_string(),
    ///                                 ..Default::default()
    ///                             },
    ///                         },
    ///                     ),
    ///                     bounding_box_target: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_hedra_character3(
        &self,
        request: &SubmitBodyHedraCharacter3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/hedra-character-3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// HiDream.ai's open-weights HiDream-O1-Image (8B): one pixel-native model that generates, edits, and personalizes without a VAE or a separate text encoder.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_hidream_o1image(
    ///             &SubmitBodyHidreamO1Image {
    ///                 input: InputHidreamO1Image {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputHidreamO1ImageAspectRatio::Sixteen9,
    ///                     images: None,
    ///                     output_format: None,
    ///                     seed: None,
    ///                     guidance_scale: None,
    ///                     num_inference_steps: None,
    ///                     resolution: None,
    ///                     quality: InputHidreamO1ImageQuality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_hidream_o1image(
        &self,
        request: &SubmitBodyHidreamO1Image,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/hidream-o1-image",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Best in class for poster-ready images and spot-on text rendering in social graphics.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_ideogram_v2(
    ///             &SubmitBodyIdeogramV2 {
    ///                 input: InputIdeogramV2 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputIdeogramV2AspectRatio::One1,
    ///                     resolution: None,
    ///                     negative_prompt: None,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_ideogram_v2(
        &self,
        request: &SubmitBodyIdeogramV2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/ideogram-v2",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Ideogram V4 renders poster-ready text and layout; the required quality parameter picks turbo, balanced or quality, which sets both the render effort and the price.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_ideogram_v4(
    ///             &SubmitBodyIdeogramV4 {
    ///                 input: InputIdeogramV4 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputIdeogramV4AspectRatio::Sixteen9,
    ///                     resolution: InputIdeogramV4Resolution::SevenHundredTwentyP,
    ///                     output_format: None,
    ///                     seed: None,
    ///                     quality: InputIdeogramV4Quality::Turbo,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_ideogram_v4(
        &self,
        request: &SubmitBodyIdeogramV4,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/ideogram-v4",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// The latest text to image model from Google
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_imagen3(
    ///             &SubmitBodyImagen3 {
    ///                 input: InputImagen3 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputImagen3AspectRatio::Sixteen9,
    ///                     resolution: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_imagen3(
        &self,
        request: &SubmitBodyImagen3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/imagen3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Google's photoreal model—natural lighting, lifelike skin, and pro-grade sharpness in every shot.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_imagen4(
    ///             &SubmitBodyImagen4 {
    ///                 input: InputImagen4 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputImagen4AspectRatio::Sixteen9,
    ///                     resolution: InputImagen4Resolution::OneK,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_imagen4(
        &self,
        request: &SubmitBodyImagen4,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/imagen4",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling16(
    ///             &SubmitBodyKling16 {
    ///                 input: InputKling16 {
    ///                     prompt: "prompt".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling16(
        &self,
        request: &SubmitBodyKling16,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-16",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Cinema-grade video with striking textures and rich depth.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling21master(
    ///             &SubmitBodyKling21Master {
    ///                 input: InputKling21Master {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputKling21MasterAspectRatio::Sixteen9,
    ///                     resolution: None,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     negative_prompt: None,
    ///                     cfg_scale: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling21master(
        &self,
        request: &SubmitBodyKling21Master,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-21-master",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Fast, high-quality video generation.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling25turbo(
    ///             &SubmitBodyKling25Turbo {
    ///                 input: InputKling25Turbo {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputKling25TurboAspectRatio::Sixteen9,
    ///                     resolution: None,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     negative_prompt: None,
    ///                     cfg_scale: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling25turbo(
        &self,
        request: &SubmitBodyKling25Turbo,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-25-turbo",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Transfer movements from a reference video to any character image. Cost-effective mode for motion transfer, perfect for portraits and simple animations.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling26motion_control(
    ///             &SubmitBodyKling26MotionControl {
    ///                 input: InputKling26MotionControl {
    ///                     num_outputs: None,
    ///                     prompt: None,
    ///                     character_orientation: None,
    ///                     start_image: InputKling26MotionControlStartImage::URL {
    ///                         data: InputKling26MotionControlStartImageURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     source_video: InputKling26MotionControlSourceVideo::URL {
    ///                         data: InputKling26MotionControlSourceVideoURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     resolution: InputKling26MotionControlResolution::SevenHundredTwentyP,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling26motion_control(
        &self,
        request: &SubmitBodyKling26MotionControl,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-26-motion-control",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Cinematic visuals, fluid motion, and native audio generation.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling26pro(
    ///             &SubmitBodyKling26Pro {
    ///                 input: InputKling26Pro {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputKling26ProAspectRatio::Sixteen9,
    ///                     resolution: None,
    ///                     duration_ms: 1,
    ///                     generate_audio: None,
    ///                     start_image: None,
    ///                     negative_prompt: None,
    ///                     cfg_scale: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling26pro(
        &self,
        request: &SubmitBodyKling26Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-26-pro",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Create avatar videos with realistic humans, animals, cartoons, or stylized characters from an image and audio input.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling_ai_avatar_v2(
    ///             &SubmitBodyKlingAiAvatarV2 {
    ///                 input: InputKlingAiAvatarV2 {
    ///                     num_outputs: None,
    ///                     prompt: None,
    ///                     aspect_ratio: InputKlingAiAvatarV2AspectRatio::Sixteen9,
    ///                     resolution: None,
    ///                     start_image: InputKlingAiAvatarV2StartImage::URL {
    ///                         data: InputKlingAiAvatarV2StartImageURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     audio: InputKlingAiAvatarV2Audio::URL {
    ///                         data: InputKlingAiAvatarV2AudioURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     quality: InputKlingAiAvatarV2Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling_ai_avatar_v2(
        &self,
        request: &SubmitBodyKlingAiAvatarV2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-ai-avatar-v2",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Generate from a single image with text-driven style and scene guidance.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling_o1(
    ///             &SubmitBodyKlingO1 {
    ///                 input: InputKlingO1 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputKlingO1AspectRatio::Sixteen9,
    ///                     resolution: None,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     images: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling_o1(
        &self,
        request: &SubmitBodyKlingO1,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-o1",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Text-to-video model with up to 15-second generations and native audio.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling_o3(
    ///             &SubmitBodyKlingO3 {
    ///                 input: InputKlingO3 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     generate_audio: None,
    ///                     aspect_ratio: InputKlingO3AspectRatio::Sixteen9,
    ///                     resolution: None,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     images: None,
    ///                     quality: InputKlingO3Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling_o3(
        &self,
        request: &SubmitBodyKlingO3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-o3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Edit videos using natural language.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling_o3edit(
    ///             &SubmitBodyKlingO3Edit {
    ///                 input: InputKlingO3Edit {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     source_video: InputKlingO3EditSourceVideo::URL {
    ///                         data: InputKlingO3EditSourceVideoURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     images: None,
    ///                     resolution: InputKlingO3EditResolution::SevenHundredTwentyP,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling_o3edit(
        &self,
        request: &SubmitBodyKlingO3Edit,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-o3-edit",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Input a reference video and preserve motion and camera style.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling_o3reference(
    ///             &SubmitBodyKlingO3Reference {
    ///                 input: InputKlingO3Reference {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     source_video: InputKlingO3ReferenceSourceVideo::URL {
    ///                         data: InputKlingO3ReferenceSourceVideoURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     images: None,
    ///                     resolution: InputKlingO3ReferenceResolution::SevenHundredTwentyP,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling_o3reference(
        &self,
        request: &SubmitBodyKlingO3Reference,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-o3-reference",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Text-to-video with ultra-high-definition storyboards and native audio.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling_v3(
    ///             &SubmitBodyKlingV3 {
    ///                 input: InputKlingV3 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     generate_audio: None,
    ///                     negative_prompt: None,
    ///                     aspect_ratio: InputKlingV3AspectRatio::Sixteen9,
    ///                     resolution: None,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     cfg_scale: None,
    ///                     end_image: None,
    ///                     quality: InputKlingV3Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling_v3(
        &self,
        request: &SubmitBodyKlingV3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-v3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Animate a character image to match the motion of a reference video. Standard tier for cost-effective generation.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_kling_v3motion_control(
    ///             &SubmitBodyKlingV3MotionControl {
    ///                 input: InputKlingV3MotionControl {
    ///                     num_outputs: None,
    ///                     prompt: None,
    ///                     character_orientation: None,
    ///                     start_image: InputKlingV3MotionControlStartImage::URL {
    ///                         data: InputKlingV3MotionControlStartImageURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     source_video: InputKlingV3MotionControlSourceVideo::URL {
    ///                         data: InputKlingV3MotionControlSourceVideoURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     resolution: InputKlingV3MotionControlResolution::SevenHundredTwentyP,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_kling_v3motion_control(
        &self,
        request: &SubmitBodyKlingV3MotionControl,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/kling-v3-motion-control",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Lightricks LTX-2.3 text-to-video at up to 4K, with synchronized native audio
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_ltx23(
    ///             &SubmitBodyLtx23 {
    ///                 input: InputLtx23 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     resolution: InputLtx23Resolution::OneThousandEightyP,
    ///                     generate_audio: None,
    ///                     duration_ms: 1,
    ///                     aspect_ratio: InputLtx23AspectRatio::Auto,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     quality: InputLtx23Quality::Fast,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_ltx23(
        &self,
        request: &SubmitBodyLtx23,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/ltx-2-3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Luma Ray 3.2 text-to-video with cinematic motion and camera control
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_luma_ray32(
    ///             &SubmitBodyLumaRay32 {
    ///                 input: InputLumaRay32 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputLumaRay32AspectRatio::One1,
    ///                     resolution: InputLumaRay32Resolution::FiveHundredFortyP,
    ///                     duration_ms: 1,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_luma_ray32(
        &self,
        request: &SubmitBodyLumaRay32,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/luma-ray-32",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Microsoft AI's MAI-Image-2.5: photorealistic generation and editing with strong in-image typography and design-ready output.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_mai_image25(
    ///             &SubmitBodyMaiImage25 {
    ///                 input: InputMaiImage25 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputMaiImage25AspectRatio::One1,
    ///                     images: None,
    ///                     output_format: None,
    ///                     quality: InputMaiImage25Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_mai_image25(
        &self,
        request: &SubmitBodyMaiImage25,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/mai-image-2-5",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// MiniMax H3 video generation from text, frames, or references.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_minimax_h3(
    ///             &SubmitBodyMinimaxH3 {
    ///                 input: InputMinimaxH3 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     resolution: InputMinimaxH3Resolution::SevenHundredSixtyEightP,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     aspect_ratio: None,
    ///                     images: None,
    ///                     videos: None,
    ///                     audios: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_minimax_h3(
        &self,
        request: &SubmitBodyMinimaxH3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/minimax-h3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Everyday 1080p video with natural movement.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_minimax_hailuo02(
    ///             &SubmitBodyMinimaxHailuo02 {
    ///                 input: InputMinimaxHailuo02 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     resolution: None,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     aspect_ratio: None,
    ///                     quality: InputMinimaxHailuo02Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_minimax_hailuo02(
        &self,
        request: &SubmitBodyMinimaxHailuo02,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/minimax-hailuo-02",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Everyday 1080p video with natural movement.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_minimax_hailuo23(
    ///             &SubmitBodyMinimaxHailuo23 {
    ///                 input: InputMinimaxHailuo23 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     resolution: None,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     aspect_ratio: None,
    ///                     quality: InputMinimaxHailuo23Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_minimax_hailuo23(
        &self,
        request: &SubmitBodyMinimaxHailuo23,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/minimax-hailuo-23",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// The brand new HD model. Ultimate Similarity, Ultra-High Quality. Supports 40+ languages including Tamil, Hebrew, Swedish, etc.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_minimax_speech25hd_preview(
    ///             &SubmitBodyMinimaxSpeech25HdPreview {
    ///                 input: InputMinimaxSpeech25HdPreview {
    ///                     text: "text".to_string(),
    ///                     voice_id: "voice_id".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_minimax_speech25hd_preview(
        &self,
        request: &SubmitBodyMinimaxSpeech25HdPreview,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/minimax-speech-25-hd-preview",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// The brand new Turbo model. Ultimate Value, 40 Languages. Major improvements to natural English expression.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_minimax_speech25turbo_preview(
    ///             &SubmitBodyMinimaxSpeech25TurboPreview {
    ///                 input: InputMinimaxSpeech25TurboPreview {
    ///                     text: "text".to_string(),
    ///                     voice_id: "voice_id".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_minimax_speech25turbo_preview(
        &self,
        request: &SubmitBodyMinimaxSpeech25TurboPreview,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/minimax-speech-25-turbo-preview",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Best in class image model with reference image support and ultra high quality generations for professional grade images.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_nano_banana(
    ///             &SubmitBodyNanoBanana {
    ///                 input: InputNanoBanana {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputNanoBananaAspectRatio::Sixteen9,
    ///                     resolution: InputNanoBananaResolution::OneK,
    ///                     images: None,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_nano_banana(
        &self,
        request: &SubmitBodyNanoBanana,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/nano-banana",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Gemini 3.1 Flash native image generation with improved quality and advanced features including multi-subject reference and high-fidelity style transfer
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_nano_banana2(
    ///             &SubmitBodyNanoBanana2 {
    ///                 input: InputNanoBanana2 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputNanoBanana2AspectRatio::Sixteen9,
    ///                     resolution: InputNanoBanana2Resolution::OneK,
    ///                     images: None,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_nano_banana2(
        &self,
        request: &SubmitBodyNanoBanana2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/nano-banana-2",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Gemini 3 Pro native image generation with advanced multimodal understanding and richer visuals
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_nano_banana_pro(
    ///             &SubmitBodyNanoBananaPro {
    ///                 input: InputNanoBananaPro {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputNanoBananaProAspectRatio::Sixteen9,
    ///                     resolution: InputNanoBananaProResolution::OneK,
    ///                     images: None,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_nano_banana_pro(
        &self,
        request: &SubmitBodyNanoBananaPro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/nano-banana-pro",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Creates vivid, emotional character videos driven entirely by your audio.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_omnihuman15(
    ///             &SubmitBodyOmnihuman15 {
    ///                 input: InputOmnihuman15 {
    ///                     num_outputs: None,
    ///                     prompt: None,
    ///                     aspect_ratio: None,
    ///                     resolution: InputOmnihuman15Resolution::SevenHundredTwentyP,
    ///                     start_image: InputOmnihuman15StartImage::URL {
    ///                         data: InputOmnihuman15StartImageURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     audio: InputOmnihuman15Audio::URL {
    ///                         data: InputOmnihuman15AudioURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_omnihuman15(
        &self,
        request: &SubmitBodyOmnihuman15,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/omnihuman-15",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// PixVerse V6 text-to-video with native audio and 1080p output up to 15 seconds
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_pixverse_v6(
    ///             &SubmitBodyPixverseV6 {
    ///                 input: InputPixverseV6 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     resolution: InputPixverseV6Resolution::ThreeHundredSixtyP,
    ///                     duration_ms: 1,
    ///                     generate_audio: None,
    ///                     negative_prompt: None,
    ///                     seed: None,
    ///                     start_image: None,
    ///                     aspect_ratio: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_pixverse_v6(
        &self,
        request: &SubmitBodyPixverseV6,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/pixverse-v6",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Alibaba's Qwen-Image-2.0, tuned for speed. Native 2K output with professional in-image text rendering, for rapid iteration.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_qwen_image2(
    ///             &SubmitBodyQwenImage2 {
    ///                 input: InputQwenImage2 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputQwenImage2AspectRatio::Sixteen9,
    ///                     resolution: InputQwenImage2Resolution::FiveHundredFortyP,
    ///                     output_format: None,
    ///                     negative_prompt: None,
    ///                     images: None,
    ///                     seed: None,
    ///                     quality: InputQwenImage2Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_qwen_image2(
        &self,
        request: &SubmitBodyQwenImage2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/qwen-image-2",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Vector-clean graphics and crisp logos on demand.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_recraft_v3(
    ///             &SubmitBodyRecraftV3 {
    ///                 input: InputRecraftV3 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputRecraftV3AspectRatio::One1,
    ///                     resolution: InputRecraftV3Resolution::FiveHundredFortyP,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_recraft_v3(
        &self,
        request: &SubmitBodyRecraftV3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/recraft-v3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Generate images from a text prompt with strong prompt adherence, layout intelligence, and accurate text rendering
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_reve21(
    ///             &SubmitBodyReve21 {
    ///                 input: InputReve21 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputReve21AspectRatio::Four1,
    ///                     output_format: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_reve21(
        &self,
        request: &SubmitBodyReve21,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/reve-21",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Edit one source image from a natural-language instruction, keeping the rest of the image intact
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_reve21edit(
    ///             &SubmitBodyReve21Edit {
    ///                 input: InputReve21Edit {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputReve21EditAspectRatio::Four1,
    ///                     images: vec![InputReve21EditImagesItem::URL {
    ///                         data: InputReve21EditImagesItemURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     }],
    ///                     output_format: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_reve21edit(
        &self,
        request: &SubmitBodyReve21Edit,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/reve-21-edit",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Compose up to eight reference images into a new image from a text prompt
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_reve21remix(
    ///             &SubmitBodyReve21Remix {
    ///                 input: InputReve21Remix {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputReve21RemixAspectRatio::Four1,
    ///                     images: vec![InputReve21RemixImagesItem::URL {
    ///                         data: InputReve21RemixImagesItemURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     }],
    ///                     output_format: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_reve21remix(
        &self,
        request: &SubmitBodyReve21Remix,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/reve-21-remix",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Lightning-fast and cheap for simple product shots or everyday content.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_sana(
    ///             &SubmitBodySana {
    ///                 input: InputSana {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputSanaAspectRatio::One1,
    ///                     resolution: InputSanaResolution::FiveHundredFortyP,
    ///                     output_format: None,
    ///                     negative_prompt: None,
    ///                     seed: None,
    ///                     guidance_scale: None,
    ///                     num_inference_steps: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_sana(
        &self,
        request: &SubmitBodySana,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/sana",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// ByteDance Seedance 1.5 Pro video generation model
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_seedance15pro(
    ///             &SubmitBodySeedance15Pro {
    ///                 input: InputSeedance15Pro {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputSeedance15ProAspectRatio::One1,
    ///                     resolution: InputSeedance15ProResolution::FourHundredEightyP,
    ///                     duration_ms: 1,
    ///                     generate_audio: None,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     seed: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_seedance15pro(
        &self,
        request: &SubmitBodySeedance15Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/seedance-15-pro",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// ByteDance Seedance 2.0 video generation model
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_seedance20(
    ///             &SubmitBodySeedance20 {
    ///                 input: InputSeedance20 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputSeedance20AspectRatio::One1,
    ///                     resolution: InputSeedance20Resolution::FourK,
    ///                     duration_ms: 1,
    ///                     generate_audio: None,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     images: None,
    ///                     videos: None,
    ///                     audios: None,
    ///                     quality: InputSeedance20Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_seedance20(
        &self,
        request: &SubmitBodySeedance20,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/seedance-20",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// ByteDance Seedance 2.0 Mini video generation model
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_seedance20mini(
    ///             &SubmitBodySeedance20Mini {
    ///                 input: InputSeedance20Mini {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputSeedance20MiniAspectRatio::One1,
    ///                     resolution: InputSeedance20MiniResolution::FourHundredEightyP,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     images: None,
    ///                     videos: None,
    ///                     audios: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_seedance20mini(
        &self,
        request: &SubmitBodySeedance20Mini,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/seedance-20-mini",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// ByteDance Seedance 2.5 video generation model
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_seedance25(
    ///             &SubmitBodySeedance25 {
    ///                 input: InputSeedance25 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputSeedance25AspectRatio::One1,
    ///                     resolution: InputSeedance25Resolution::FourHundredEightyP,
    ///                     duration_ms: 1,
    ///                     generate_audio: None,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     images: None,
    ///                     videos: None,
    ///                     audios: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_seedance25(
        &self,
        request: &SubmitBodySeedance25,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/seedance-25",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Ultra-fast pro grade image model, pairing reference image support with high quality output for professional visuals
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_seedream40(
    ///             &SubmitBodySeedream40 {
    ///                 input: InputSeedream40 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputSeedream40AspectRatio::One1,
    ///                     resolution: InputSeedream40Resolution::OneThousandEightyP,
    ///                     images: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_seedream40(
        &self,
        request: &SubmitBodySeedream40,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/seedream-40",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Latest Seedream with enhanced detail, refined composition, and multi-reference image support for professional visuals.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_seedream45(
    ///             &SubmitBodySeedream45 {
    ///                 input: InputSeedream45 {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputSeedream45AspectRatio::One1,
    ///                     resolution: InputSeedream45Resolution::OneThousandFourHundredFortyP2KQhd,
    ///                     images: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_seedream45(
        &self,
        request: &SubmitBodySeedream45,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/seedream-45",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// ByteDance Seedream 5.0 Lite Text-to-Image
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_seedream50lite(
    ///             &SubmitBodySeedream50Lite {
    ///                 input: InputSeedream50Lite {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputSeedream50LiteAspectRatio::One1,
    ///                     resolution: InputSeedream50LiteResolution::OneThousandFourHundredFortyP2KQhd,
    ///                     images: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_seedream50lite(
        &self,
        request: &SubmitBodySeedream50Lite,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/seedream-50-lite",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// ByteDance Seedream 5.0 Pro Text-to-Image
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_seedream50pro(
    ///             &SubmitBodySeedream50Pro {
    ///                 input: InputSeedream50Pro {
    ///                     prompt: "prompt".to_string(),
    ///                     num_outputs: None,
    ///                     enhance_prompt: None,
    ///                     aspect_ratio: InputSeedream50ProAspectRatio::One1,
    ///                     resolution: InputSeedream50ProResolution::OneThousandEightyP,
    ///                     images: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_seedream50pro(
        &self,
        request: &SubmitBodySeedream50Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/seedream-50-pro",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// For complex, narrative-driven videos with remarkable consistency and realistic character-world interaction.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_sora2pro(
    ///             &SubmitBodySora2Pro {
    ///                 input: InputSora2Pro {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputSora2ProAspectRatio::Sixteen9,
    ///                     resolution: InputSora2ProResolution::SevenHundredTwentyP,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_sora2pro(
        &self,
        request: &SubmitBodySora2Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/sora-2-pro",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Talking video with natural lip-sync and expressive animation.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_veed_fabric10(
    ///             &SubmitBodyVeedFabric10 {
    ///                 input: InputVeedFabric10 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputVeedFabric10AspectRatio::Sixteen9,
    ///                     resolution: InputVeedFabric10Resolution::FourHundredEightyP,
    ///                     start_image: InputVeedFabric10StartImage::URL {
    ///                         data: InputVeedFabric10StartImageURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                     audio: InputVeedFabric10Audio::URL {
    ///                         data: InputVeedFabric10AudioURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     },
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_veed_fabric10(
        &self,
        request: &SubmitBodyVeedFabric10,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/veed-fabric-10",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// The current state of the art in video generation
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_veo2(
    ///             &SubmitBodyVeo2 {
    ///                 input: InputVeo2 {
    ///                     prompt: "prompt".to_string(),
    ///                     ..Default::default()
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_veo2(
        &self,
        request: &SubmitBodyVeo2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/veo-2",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Hollywood-grade, cinematic video straight from text—your go-to for hero campaigns.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_veo3(
    ///             &SubmitBodyVeo3 {
    ///                 input: InputVeo3 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputVeo3AspectRatio::Sixteen9,
    ///                     resolution: InputVeo3Resolution::SevenHundredTwentyP,
    ///                     duration_ms: 1,
    ///                     generate_audio: None,
    ///                     start_image: None,
    ///                     negative_prompt: None,
    ///                     seed: None,
    ///                     quality: InputVeo3Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_veo3(
        &self,
        request: &SubmitBodyVeo3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/veo-3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// For unparalleled detail and nuance, perfect for when your vision requires the best possible quality.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_veo31(
    ///             &SubmitBodyVeo31 {
    ///                 input: InputVeo31 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     aspect_ratio: InputVeo31AspectRatio::Sixteen9,
    ///                     resolution: InputVeo31Resolution::SevenHundredTwentyP,
    ///                     duration_ms: None,
    ///                     generate_audio: None,
    ///                     negative_prompt: None,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     source_video: None,
    ///                     images: None,
    ///                     seed: None,
    ///                     quality: InputVeo31Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_veo31(
        &self,
        request: &SubmitBodyVeo31,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/veo-31",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Vidu Q3 video with native dialogue and sound, up to 16 seconds — from a text prompt, from a start frame, or between a start and end frame
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_vidu_q3(
    ///             &SubmitBodyViduQ3 {
    ///                 input: InputViduQ3 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     seed: None,
    ///                     resolution: InputViduQ3Resolution::FiveHundredFortyP,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     aspect_ratio: None,
    ///                     quality: InputViduQ3Quality::Standard,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_vidu_q3(
        &self,
        request: &SubmitBodyViduQ3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/vidu-q3",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Vidu Q3 reference-to-video keeping up to four subjects consistent
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_vidu_q3reference(
    ///             &SubmitBodyViduQ3Reference {
    ///                 input: InputViduQ3Reference {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     seed: None,
    ///                     aspect_ratio: InputViduQ3ReferenceAspectRatio::Sixteen9,
    ///                     resolution: InputViduQ3ReferenceResolution::FiveHundredFortyP,
    ///                     duration_ms: 1,
    ///                     images: vec![InputViduQ3ReferenceImagesItem::URL {
    ///                         data: InputViduQ3ReferenceImagesItemURL {
    ///                             url: "url".to_string(),
    ///                             ..Default::default()
    ///                         },
    ///                     }],
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_vidu_q3reference(
        &self,
        request: &SubmitBodyViduQ3Reference,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/vidu-q3-reference",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Wan 2.7 video with native audio — from a text prompt, from a first frame with an optional last frame, or from reference images that keep subjects consistent
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
    ///
    /// # Arguments
    ///
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
    ///         .jobs
    ///         .submit_wan27(
    ///             &SubmitBodyWan27 {
    ///                 input: InputWan27 {
    ///                     num_outputs: None,
    ///                     prompt: "prompt".to_string(),
    ///                     negative_prompt: None,
    ///                     seed: None,
    ///                     resolution: InputWan27Resolution::SevenHundredTwentyP,
    ///                     duration_ms: 1,
    ///                     start_image: None,
    ///                     end_image: None,
    ///                     aspect_ratio: None,
    ///                     images: None,
    ///                 },
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit_wan27(
        &self,
        request: &SubmitBodyWan27,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                "models/wan-2-7",
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }

    /// Runs any model in the catalog by its public id, with `input` passed through untyped — the same call the typed operations below make, minus the compile-time schema.
    ///
    /// Reach for it when the model is not known ahead of time: a client generated before a model shipped can still run it, and an id read from `GET /v3/models` at runtime needs no regeneration. Prefer the typed operation whenever your client already has one — `input` here is validated against the same published schema (`GET /v3/models/{model}`), so a bad field is a `400` at submit rather than an error before the call.
    ///
    /// Submits an asynchronous job and returns `202` with a job id. Fetch the result at `GET /v3/jobs/{job_id}` — each item in its `outputs[]` follows the `OutputItem` schema — or track progress via `GET /v3/jobs/{job_id}/status` / the SSE stream at `GET /v3/jobs/{job_id}/stream`.
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
    ///     client
    ///         .jobs
    ///         .submit(
    ///             &"model".to_string(),
    ///             &SubmitBody {
    ///                 input: HashMap::from([("key".to_string(), serde_json::json!("value"))]),
    ///                 webhook: None,
    ///                 idempotency_key: None,
    ///             },
    ///             None,
    ///         )
    ///         .await;
    /// }
    /// ```
    pub async fn submit(
        &self,
        model: &str,
        request: &SubmitBody,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
        let options = {
            let mut o = options.unwrap_or_default();
            o.additional_headers
                .entry("X-Hedra-Spec-Version".to_string())
                .or_insert_with(|| "3.2.2".to_string());
            Some(o)
        };
        self.http_client
            .execute_request(
                Method::POST,
                &format!("models/{}", model),
                Some(serde_json::to_value(request).map_err(ApiError::Serialization)?),
                None,
                options,
            )
            .await
    }
}

use crate::api::*;
use crate::{ApiError, ClientConfig, HttpClient, QueryBuilder, RequestOptions};
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

    pub async fn list(
        &self,
        request: &JobsListQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<JobListResponse, ApiError> {
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

    pub async fn get(
        &self,
        job_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<ResultResponse, ApiError> {
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

    pub async fn get_status(
        &self,
        job_id: &str,
        request: &GetStatusQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<StatusResponse, ApiError> {
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

    pub async fn list_job_logs(
        &self,
        job_id: &str,
        request: &ListJobLogsQueryRequest,
        options: Option<RequestOptions>,
    ) -> Result<JobLogListResponse, ApiError> {
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

    pub async fn stream(
        &self,
        job_id: &str,
        options: Option<RequestOptions>,
    ) -> Result<serde_json::Value, ApiError> {
        self.http_client
            .execute_request(
                Method::GET,
                &format!("jobs/{}/stream", job_id),
                None,
                None,
                options,
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
    pub async fn submit_dreamina31(
        &self,
        request: &SubmitBodyDreamina31,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_elevenlabs_flash_multilingual_v2(
        &self,
        request: &SubmitBodyElevenlabsFlashMultilingualV2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_elevenlabs_flash_v2(
        &self,
        request: &SubmitBodyElevenlabsFlashV2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_elevenlabs_multilingual_v2(
        &self,
        request: &SubmitBodyElevenlabsMultilingualV2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_elevenlabs_v3(
        &self,
        request: &SubmitBodyElevenlabsV3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_flux11pro(
        &self,
        request: &SubmitBodyFlux11Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_flux11ultra(
        &self,
        request: &SubmitBodyFlux11Ultra,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_flux_dev(
        &self,
        request: &SubmitBodyFluxDev,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_flux_kontext_max(
        &self,
        request: &SubmitBodyFluxKontextMax,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_flux_kontext_pro(
        &self,
        request: &SubmitBodyFluxKontextPro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_flux2flex(
        &self,
        request: &SubmitBodyFlux2Flex,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_flux2klein9b(
        &self,
        request: &SubmitBodyFlux2Klein9B,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_flux2max(
        &self,
        request: &SubmitBodyFlux2Max,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_flux2pro(
        &self,
        request: &SubmitBodyFlux2Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_gemini_omni_flash(
        &self,
        request: &SubmitBodyGeminiOmniFlash,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_gpt_image15(
        &self,
        request: &SubmitBodyGptImage15,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_gpt_image2(
        &self,
        request: &SubmitBodyGptImage2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_grok_imagine(
        &self,
        request: &SubmitBodyGrokImagine,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_grok_video(
        &self,
        request: &SubmitBodyGrokVideo,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_happy_horse(
        &self,
        request: &SubmitBodyHappyHorse,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_hedra_avatar(
        &self,
        request: &SubmitBodyHedraAvatar,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_hedra_character3(
        &self,
        request: &SubmitBodyHedraCharacter3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_hidream_o1image(
        &self,
        request: &SubmitBodyHidreamO1Image,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_ideogram_v2(
        &self,
        request: &SubmitBodyIdeogramV2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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

    /// Ideogram V4 at its middle render setting; poster-ready text and layout at everyday cost.
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
    pub async fn submit_ideogram_v4(
        &self,
        request: &SubmitBodyIdeogramV4,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_imagen3(
        &self,
        request: &SubmitBodyImagen3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_imagen4(
        &self,
        request: &SubmitBodyImagen4,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_kling16(
        &self,
        request: &SubmitBodyKling16,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_kling21master(
        &self,
        request: &SubmitBodyKling21Master,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_kling25turbo(
        &self,
        request: &SubmitBodyKling25Turbo,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_kling26pro(
        &self,
        request: &SubmitBodyKling26Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_kling_ai_avatar_v2(
        &self,
        request: &SubmitBodyKlingAiAvatarV2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_kling_o1(
        &self,
        request: &SubmitBodyKlingO1,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_kling_o3(
        &self,
        request: &SubmitBodyKlingO3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_kling_v3(
        &self,
        request: &SubmitBodyKlingV3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_ltx23(
        &self,
        request: &SubmitBodyLtx23,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_luma_ray32(
        &self,
        request: &SubmitBodyLumaRay32,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_mai_image25(
        &self,
        request: &SubmitBodyMaiImage25,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_minimax_hailuo02(
        &self,
        request: &SubmitBodyMinimaxHailuo02,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_minimax_hailuo23(
        &self,
        request: &SubmitBodyMinimaxHailuo23,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_minimax_speech25hd_preview(
        &self,
        request: &SubmitBodyMinimaxSpeech25HdPreview,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_minimax_speech25turbo_preview(
        &self,
        request: &SubmitBodyMinimaxSpeech25TurboPreview,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_nano_banana(
        &self,
        request: &SubmitBodyNanoBanana,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_nano_banana2(
        &self,
        request: &SubmitBodyNanoBanana2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_nano_banana_pro(
        &self,
        request: &SubmitBodyNanoBananaPro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_omnihuman15(
        &self,
        request: &SubmitBodyOmnihuman15,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_pixverse_v6(
        &self,
        request: &SubmitBodyPixverseV6,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_qwen_image2(
        &self,
        request: &SubmitBodyQwenImage2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_recraft_v3(
        &self,
        request: &SubmitBodyRecraftV3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_reve21(
        &self,
        request: &SubmitBodyReve21,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_reve21edit(
        &self,
        request: &SubmitBodyReve21Edit,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_reve21remix(
        &self,
        request: &SubmitBodyReve21Remix,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_sana(
        &self,
        request: &SubmitBodySana,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_seedance15pro(
        &self,
        request: &SubmitBodySeedance15Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_seedance20(
        &self,
        request: &SubmitBodySeedance20,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_seedance20mini(
        &self,
        request: &SubmitBodySeedance20Mini,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_seedream40(
        &self,
        request: &SubmitBodySeedream40,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_seedream45(
        &self,
        request: &SubmitBodySeedream45,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_seedream50lite(
        &self,
        request: &SubmitBodySeedream50Lite,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_seedream50pro(
        &self,
        request: &SubmitBodySeedream50Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_sora2pro(
        &self,
        request: &SubmitBodySora2Pro,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_veed_fabric10(
        &self,
        request: &SubmitBodyVeedFabric10,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_veo2(
        &self,
        request: &SubmitBodyVeo2,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_veo3(
        &self,
        request: &SubmitBodyVeo3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_veo31(
        &self,
        request: &SubmitBodyVeo31,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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

    /// Vidu Q3 text-to-video with native dialogue and sound, up to 16 seconds
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
    pub async fn submit_vidu_q3(
        &self,
        request: &SubmitBodyViduQ3,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
    pub async fn submit_vidu_q3reference(
        &self,
        request: &SubmitBodyViduQ3Reference,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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

    /// Wan 2.7 text-to-video with native audio and up to 15-second generations
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
    pub async fn submit_wan27(
        &self,
        request: &SubmitBodyWan27,
        options: Option<RequestOptions>,
    ) -> Result<SubmitResponse, ApiError> {
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
}
